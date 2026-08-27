//! Step 2 — in-memory `AsyncBookmarkStore` (`src/in_memory.rs`).
//!
//! Unlock: delete the `#[ignore]` on each test as you implement it, or run all
//! at once with `cargo test --test step_02_in_memory -- --ignored`.
//!
//! Java/Spring: exercising a `@Repository` backed by a ConcurrentHashMap — async
//! signatures, but the bodies are pure data-structure work.

use readlater::in_memory::InMemoryBookmarkStore;
use readlater::model::{Bookmark, BookmarkStatus};
use readlater::store::{AsyncBookmarkStore, ListQuery};

fn bm(id: &str, url: &str, tags: &[&str], created_at: i64) -> Bookmark {
    Bookmark::new(
        id.to_string(),
        url.to_string(),
        tags.iter().map(|s| s.to_string()).collect(),
        created_at,
    )
}

#[tokio::test]
async fn insert_then_get_returns_the_bookmark() {
    let store = InMemoryBookmarkStore::new();
    store
        .insert(bm("a", "https://a.test", &[], 1))
        .await
        .unwrap();

    let got = store.get("a").await.unwrap().expect("present");
    assert_eq!(got.url, "https://a.test");
    assert!(store.get("missing").await.unwrap().is_none());
}

#[tokio::test]
async fn list_is_newest_first_and_filters_compose() {
    let store = InMemoryBookmarkStore::new();
    store
        .insert(bm("old", "https://old.test", &["rust"], 1))
        .await
        .unwrap();
    store
        .insert(bm("new", "https://new.test", &["rust", "web"], 2))
        .await
        .unwrap();

    // Newest first.
    let all = store.list(&ListQuery::default()).await.unwrap();
    assert_eq!(
        all.iter().map(|b| b.id.as_str()).collect::<Vec<_>>(),
        ["new", "old"]
    );

    // tag filter.
    let web = store
        .list(&ListQuery {
            tag: Some("web".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(web.len(), 1);
    assert_eq!(web[0].id, "new");

    // read filter: nothing is read yet.
    let read_only = store
        .list(&ListQuery {
            read: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(read_only.is_empty());
}

#[tokio::test]
async fn set_read_and_delete_report_existence() {
    let store = InMemoryBookmarkStore::new();
    store
        .insert(bm("a", "https://a.test", &[], 1))
        .await
        .unwrap();

    assert!(store.set_read("a", true).await.unwrap());
    assert!(!store.set_read("nope", true).await.unwrap());
    assert!(store.get("a").await.unwrap().unwrap().read);

    assert!(store.delete("a").await.unwrap());
    assert!(!store.delete("a").await.unwrap());
}

#[tokio::test]
async fn mark_ready_and_failed_transition_status() {
    let store = InMemoryBookmarkStore::new();
    store
        .insert(bm("a", "https://a.test", &[], 1))
        .await
        .unwrap();
    store
        .insert(bm("b", "https://b.test", &[], 2))
        .await
        .unwrap();

    store
        .mark_ready("a", Some("Title".into()), Some("Excerpt".into()))
        .await
        .unwrap();
    let a = store.get("a").await.unwrap().unwrap();
    assert_eq!(a.status, BookmarkStatus::Ready);
    assert_eq!(a.title.as_deref(), Some("Title"));
    assert_eq!(a.error, None);

    store.mark_failed("b", "boom".into()).await.unwrap();
    let b = store.get("b").await.unwrap().unwrap();
    assert_eq!(b.status, BookmarkStatus::Failed);
    assert_eq!(b.error.as_deref(), Some("boom"));
}
