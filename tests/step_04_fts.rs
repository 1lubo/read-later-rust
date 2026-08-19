//! Step 4 — full-text search over title + excerpt (`src/sqlite.rs`, FTS5).
//!
//! Offline temp-file SQLite. Unlock with
//! `cargo test --test step_04_fts -- --ignored`.
//!
//! Java/Spring: like adding a Hibernate Search / a `@Query` with a MATCH — the
//! store keeps a separate FTS index in sync and `?q=` queries it.

use readlater::model::Bookmark;
use readlater::store::{AsyncBookmarkStore, ListQuery};
use readlater::sqlite::SqliteBookmarkStore;

async fn fresh_store() -> (SqliteBookmarkStore, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("bookmarks.db");
    let url = format!("sqlite:{}?mode=rwc", path.display());
    let store = SqliteBookmarkStore::connect(&url).await.expect("connect");
    store.migrate().await.expect("migrate");
    (store, tmp)
}

fn bm(id: &str, url: &str) -> Bookmark {
    Bookmark::new(id.to_string(), url.to_string(), vec![], 1)
}

#[tokio::test]
#[ignore = "step 4: keep bookmarks_fts in sync and query it for ?q="]
async fn search_matches_title_and_excerpt() {
    let (store, _tmp) = fresh_store().await;

    store.insert(bm("rust", "https://rust.test")).await.unwrap();
    store
        .mark_ready("rust", Some("Learning Rust".into()), Some("ownership and borrowing".into()))
        .await
        .unwrap();

    store.insert(bm("java", "https://java.test")).await.unwrap();
    store
        .mark_ready("java", Some("Spring Boot".into()), Some("dependency injection".into()))
        .await
        .unwrap();

    // Match on title.
    let hits = store
        .list(&ListQuery { q: Some("rust".into()), ..Default::default() })
        .await
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "rust");

    // Match on excerpt.
    let hits = store
        .list(&ListQuery { q: Some("injection".into()), ..Default::default() })
        .await
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "java");

    // No match.
    let none = store
        .list(&ListQuery { q: Some("kubernetes".into()), ..Default::default() })
        .await
        .unwrap();
    assert!(none.is_empty());
}

#[tokio::test]
#[ignore = "step 4: deleting a bookmark must also drop its FTS row"]
async fn delete_removes_from_search_index() {
    let (store, _tmp) = fresh_store().await;
    store.insert(bm("rust", "https://rust.test")).await.unwrap();
    store.mark_ready("rust", Some("Learning Rust".into()), None).await.unwrap();

    assert_eq!(
        store.list(&ListQuery { q: Some("rust".into()), ..Default::default() }).await.unwrap().len(),
        1
    );

    store.delete("rust").await.unwrap();
    assert!(store
        .list(&ListQuery { q: Some("rust".into()), ..Default::default() })
        .await
        .unwrap()
        .is_empty());
}
