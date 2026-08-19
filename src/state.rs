//! Shared application state.
//!
//! Java/Spring: `AppState` is the hand-wired equivalent of the Spring
//! `ApplicationContext` beans a controller depends on. Axum hands a clone of
//! this to every handler via the `State` extractor, so it MUST be cheap to
//! `Clone` (each field here is: an enum wrapping an `Arc`/pool, an mpsc
//! `Sender`, and an `AuthConfig` whose fields are `Clone`).

use tokio::sync::mpsc;

use crate::auth::AuthConfig;
use crate::store::StoreHandle;

/// The dependency bundle injected into every handler.
#[derive(Clone)]
pub struct AppState {
    /// The chosen storage backend (in-memory in tests, SQLite in prod).
    pub store: StoreHandle,
    /// Enqueue a bookmark id for the background enrichment worker.
    /// Java/Spring: handing a message to a `@Async` method / a queue producer.
    pub enqueue: mpsc::Sender<String>,
    /// Token + cookie-signing key used by the auth extractor.
    pub auth: AuthConfig,
}

impl AppState {
    pub fn new(store: StoreHandle, enqueue: mpsc::Sender<String>, auth: AuthConfig) -> Self {
        Self {
            store,
            enqueue,
            auth,
        }
    }
}
