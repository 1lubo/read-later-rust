//! Step 3 — SQLite `AsyncBookmarkStore` (`src/sqlite.rs`).
//!
//! Offline: each test opens its OWN throwaway on-disk SQLite file (no Docker,
//! no network). Unlock with `cargo test --test step_03_sqlite -- --ignored`.
//!
//! Java/Spring: swapping the in-memory repo for a JDBC/JPA one — the SAME store
//! contract, now backed by real SQL + Flyway-style migrations.

use readlater::model::{Bookmark, BookmarkStatus};
use readlater::sqlite::SqliteBookmarkStore;
use readlater::store::{AsyncBookmarkStore, ListQuery};

/// Open a migrated store over a fresh temp file. `_tmp` must stay in scope so
/// the file isn't deleted while the pool is open — so we return it.
async fn fresh_store() -> (SqliteBookmarkStore, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("bookmarks.db");
    let url = format!("sqlite:{}?mode=rwc", path.display());
    let store = SqliteBookmarkStore::connect(&url).await.expect("connect");
    store.migrate().await.expect("migrate");
    (store, tmp)
}

fn bm(id: &str, url: &str, tags: &[&str], created_at: i64) -> Bookmark {
    Bookmark::new(
        id.to_string(),
        url.to_string(),
        tags.iter().map(|s| s.to_string()).collect(),
        created_at,
    )
}

#[tokio::test]
async fn insert_get_roundtrips_with_tags() {
    let (store, _tmp) = fresh_store().await;
    store
        .insert(bm("a", "https://a.test", &["rust", "web"], 1))
        .await
        .unwrap();

    let got = store.get("a").await.unwrap().expect("present");
    assert_eq!(got.url, "https://a.test");
    assert_eq!(got.status, BookmarkStatus::Pending);
    let mut tags = got.tags.clone();
    tags.sort();
    assert_eq!(tags, vec!["rust".to_string(), "web".to_string()]);
    assert!(store.get("missing").await.unwrap().is_none());
}

#[tokio::test]
async fn list_orders_and_filters() {
    let (store, _tmp) = fresh_store().await;
    store
        .insert(bm("old", "https://old.test", &["rust"], 1))
        .await
        .unwrap();
    store
        .insert(bm("new", "https://new.test", &["rust", "web"], 2))
        .await
        .unwrap();

    let all = store.list(&ListQuery::default()).await.unwrap();
    assert_eq!(
        all.iter().map(|b| b.id.as_str()).collect::<Vec<_>>(),
        ["new", "old"]
    );

    let web = store
        .list(&ListQuery {
            tag: Some("web".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(web.len(), 1);
    assert_eq!(web[0].id, "new");
}

#[tokio::test]
async fn mutations_persist_and_report_existence() {
    let (store, _tmp) = fresh_store().await;
    store
        .insert(bm("a", "https://a.test", &[], 1))
        .await
        .unwrap();

    assert!(store.set_read("a", true).await.unwrap());
    assert!(!store.set_read("nope", true).await.unwrap());

    store
        .mark_ready("a", Some("T".into()), Some("E".into()))
        .await
        .unwrap();
    let a = store.get("a").await.unwrap().unwrap();
    assert_eq!(a.status, BookmarkStatus::Ready);
    assert!(a.read);
    assert_eq!(a.title.as_deref(), Some("T"));

    assert!(store.delete("a").await.unwrap());
    assert!(!store.delete("a").await.unwrap());
}
