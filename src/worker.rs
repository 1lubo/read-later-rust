//! Background enrichment worker (step 6).
//!
//! Java/Spring: think of a `@Component` consuming from a queue — an `@Async`
//! listener that, given a bookmark id, fetches the page and updates the row.
//! Here the "queue" is a Tokio `mpsc` channel and the "listener" is `run`,
//! which loops on `rx.recv()`. The generic `Worker<S, F>` is closed over the
//! two seams (`AsyncBookmarkStore`, `PageFetcher`) so tests drive it with fakes.

use tokio::sync::mpsc;

use crate::error::AppError;
use crate::fetcher::PageFetcher;
use crate::store::AsyncBookmarkStore;

/// Owns a store + fetcher and turns a `Pending` bookmark into `Ready`/`Failed`.
pub struct Worker<S, F> {
    store: S,
    fetcher: F,
}

impl<S, F> Worker<S, F>
where
    S: AsyncBookmarkStore,
    F: PageFetcher,
{
    pub fn new(store: S, fetcher: F) -> Self {
        Self { store, fetcher }
    }

    /// Enrich exactly one bookmark. Public so a test can drive a single,
    /// deterministic enrichment with a `FakeFetcher` (no channel, no sleeps).
    ///
    /// Steps:
    /// 1. `self.store.get(id)` — if absent, just return `Ok(())` (nothing to do).
    /// 2. `self.fetcher.fetch(&bookmark.url).await`.
    /// 3. On `Ok(meta)` -> `self.store.mark_ready(id, meta.title, meta.excerpt)`.
    /// 4. On `Err(e)` -> `self.store.mark_failed(id, e.to_string())`.
    ///
    /// Java/Spring: the body of the `@Async` "process one message" method.
    pub async fn process(&self, id: &str) -> Result<(), AppError> {
        let Some(bookmark) = self.store.get(id).await? else {
            return Ok(());
        };

        match self.fetcher.fetch(&bookmark.url).await {
            Ok(meta) => self.store.mark_ready(id, meta.title, meta.excerpt).await,
            Err(e) => self.store.mark_failed(id, e.to_string()).await,
        }
    }

    /// The consumer loop. (Provided plumbing — spawned as a Tokio task in
    /// `main`.) Runs until every `Sender` is dropped and the channel closes.
    pub async fn run(self, mut rx: mpsc::Receiver<String>) {
        while let Some(id) = rx.recv().await {
            if let Err(e) = self.process(&id).await {
                tracing::warn!(bookmark_id = %id, error = %e, "enrichment failed");
            }
        }
    }
}
