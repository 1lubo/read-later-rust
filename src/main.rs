//! Binary entrypoint (step 10): read config, open + migrate the DB, spawn the
//! background worker, merge the JSON + web routers, and serve.
//!
//! Java/Spring: this is your `@SpringBootApplication main()` PLUS the manual
//! wiring Spring's auto-configuration would otherwise do — there is no magic
//! component scan, so every dependency is constructed and connected by hand
//! here. That explicitness is the point: you can see the whole object graph.

use std::net::SocketAddr;

use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use readlater::api::api_router;
use readlater::auth::AuthConfig;
use readlater::config::Config;
use readlater::fetcher::ReqwestFetcher;
use readlater::sqlite::SqliteBookmarkStore;
use readlater::state::AppState;
use readlater::store::StoreHandle;
use readlater::web::web_router;
use readlater::worker::Worker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Structured logging (provided). Java/Spring: Logback auto-config.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    // 1. Config from the environment.
    let config = Config::from_env().map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    // 2. Open + migrate the SQLite store.
    let store = SqliteBookmarkStore::connect(&config.database_url).await?;
    store.migrate().await?;

    // 3. The enrichment channel. Java/Spring: a bounded work queue.
    let (tx, rx) = mpsc::channel::<String>(128);

    // 4. Spawn the background worker (SQLite store + real HTTP fetcher).
    let worker = Worker::new(StoreHandle::Sqlite(store.clone()), ReqwestFetcher::new());
    tokio::spawn(worker.run(rx));

    // 5. Build shared state and the combined router.
    let auth = AuthConfig::new(config.token.clone(), &config.cookie_key);
    let state = AppState::new(StoreHandle::Sqlite(store), tx, auth);
    let app = api_router().merge(web_router()).with_state(state);

    // 6. Serve. Java/Spring: the embedded Tomcat listening on server.port.
    let addr: SocketAddr = config.bind_addr.parse()?;
    tracing::info!(%addr, "readlater listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
