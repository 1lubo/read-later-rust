//! Shared test harness for the HTTP steps (8 & 9).
//!
//! Java/Spring: this is your `@WebMvcTest` setup + `MockMvc` builder. We wire an
//! `AppState` over an in-memory store (no SQLite, no network), merge the two
//! routers, and hand back the `Router` plus the `Sender` end of the enrichment
//! channel so a test can assert that a save enqueued work.
//!
//! `#![allow(dead_code)]`: not every step's test file uses every helper, and
//! each integration test binary compiles this module independently.
#![allow(dead_code)]

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response};
use http_body_util::BodyExt;
use tokio::sync::mpsc;
use tower::ServiceExt; // for `oneshot`

use readlater::api::api_router;
use readlater::auth::AuthConfig;
use readlater::in_memory::InMemoryBookmarkStore;
use readlater::state::AppState;
use readlater::store::StoreHandle;
use readlater::web::web_router;

pub const TOKEN: &str = "test-token";
const COOKIE_KEY: &[u8] = &[9u8; 64];

/// A ready-to-drive app plus the receiver a test can drain to see enqueued ids.
pub struct TestApp {
    pub router: Router,
    pub store: InMemoryBookmarkStore,
    pub enqueued: mpsc::Receiver<String>,
}

/// Build the full app (API + web) over an in-memory store. Mirrors `main`'s
/// wiring minus the SQLite/worker/serve parts.
pub fn test_app() -> TestApp {
    let store = InMemoryBookmarkStore::new();
    let (tx, rx) = mpsc::channel::<String>(64);
    let auth = AuthConfig::new(TOKEN.to_string(), COOKIE_KEY);
    let state = AppState::new(StoreHandle::Memory(store.clone()), tx, auth);
    let router = api_router().merge(web_router()).with_state(state);
    TestApp {
        router,
        store,
        enqueued: rx,
    }
}

/// Fire a single request through the router (MockMvc `perform`). Consumes the
/// router clone so callers pass `app.router.clone()`.
pub async fn send(router: Router, req: Request<Body>) -> Response<Body> {
    router.oneshot(req).await.expect("router oneshot")
}

/// Read a response body to a UTF-8 string.
pub async fn body_string(resp: Response<Body>) -> String {
    let bytes = resp
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("utf8 body")
}

/// A `Bearer` auth header value for the JSON API.
pub fn bearer() -> String {
    format!("Bearer {TOKEN}")
}
