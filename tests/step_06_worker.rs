//! Step 6 — background enrichment (`Worker::process` in `src/worker.rs`).
//!
//! Offline + deterministic: an in-memory store + a `FakeFetcher` with canned
//! responses; we call `process(id)` directly (no channel, no sleeps). Unlock
//! with `cargo test --test step_06_worker -- --ignored`.
//!
//! Java/Spring: unit-testing the body of an `@Async` "process one message"
//! method with a mocked downstream client.

use readlater::fetcher::FakeFetcher;
use readlater::in_memory::InMemoryBookmarkStore;
use readlater::model::{Bookmark, BookmarkStatus};
use readlater::store::AsyncBookmarkStore;
use readlater::worker::Worker;

fn bm(id: &str, url: &str) -> Bookmark {
    Bookmark::new(id.to_string(), url.to_string(), vec![], 1)
}

#[tokio::test]
async fn process_marks_ready_on_success() {
    let store = InMemoryBookmarkStore::new();
    store.insert(bm("a", "https://a.test")).await.unwrap();

    let fetcher = FakeFetcher::new().with("https://a.test", Some("Title"), Some("Excerpt"));
    let worker = Worker::new(store.clone(), fetcher);

    worker.process("a").await.unwrap();

    let a = store.get("a").await.unwrap().unwrap();
    assert_eq!(a.status, BookmarkStatus::Ready);
    assert_eq!(a.title.as_deref(), Some("Title"));
    assert_eq!(a.excerpt.as_deref(), Some("Excerpt"));
}

#[tokio::test]
async fn process_marks_failed_on_fetch_error() {
    let store = InMemoryBookmarkStore::new();
    store.insert(bm("a", "https://a.test")).await.unwrap();

    let fetcher = FakeFetcher::new().failing("https://a.test");
    let worker = Worker::new(store.clone(), fetcher);

    worker.process("a").await.unwrap();

    let a = store.get("a").await.unwrap().unwrap();
    assert_eq!(a.status, BookmarkStatus::Failed);
    assert!(a.error.is_some());
}

#[tokio::test]
async fn process_unknown_id_is_ok() {
    let store = InMemoryBookmarkStore::new();
    let fetcher = FakeFetcher::new();
    let worker = Worker::new(store.clone(), fetcher);

    // No bookmark with this id — should simply do nothing and succeed.
    worker.process("ghost").await.unwrap();
    assert!(store.get("ghost").await.unwrap().is_none());
}
