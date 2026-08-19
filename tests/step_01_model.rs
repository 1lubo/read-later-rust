//! Step 1 — domain model. THE ONLY ACTIVE STEP ON A FRESH CLONE.
//!
//! Run: `cargo test --test step_01_model`
//! Implement `BookmarkStatus::as_str` and `Bookmark::new` in `src/model.rs`
//! until these pass. Then unlock step 2 by deleting its `#[ignore]` line.
//!
//! Java/Spring: these are plain JUnit tests over a POJO + enum — no Spring
//! context, no HTTP. Just construct and assert.

use readlater::model::{Bookmark, BookmarkStatus};

#[test]
fn status_as_str_maps_each_variant_to_lowercase() {
    // Java: assertEquals("pending", BookmarkStatus.PENDING.value());
    assert_eq!(BookmarkStatus::Pending.as_str(), "pending");
    assert_eq!(BookmarkStatus::Ready.as_str(), "ready");
    assert_eq!(BookmarkStatus::Failed.as_str(), "failed");
}

#[test]
fn new_bookmark_starts_pending_and_unread() {
    let b = Bookmark::new(
        "id-1".to_string(),
        "https://example.com".to_string(),
        vec!["rust".to_string(), "web".to_string()],
        1_700_000_000_000,
    );

    assert_eq!(b.id, "id-1");
    assert_eq!(b.url, "https://example.com");
    assert_eq!(b.status, BookmarkStatus::Pending);
    assert_eq!(b.title, None);
    assert_eq!(b.excerpt, None);
    assert_eq!(b.error, None);
    assert!(!b.read);
    assert_eq!(b.tags, vec!["rust".to_string(), "web".to_string()]);
    assert_eq!(b.created_at, 1_700_000_000_000);
}

#[test]
fn bookmark_serializes_status_as_lowercase_string() {
    // Java: Jackson writes {"status":"pending", ...}. serde does the same via
    // #[serde(rename_all = "lowercase")].
    let b = Bookmark::new("id-2".to_string(), "https://x.test".to_string(), vec![], 0);
    let json = serde_json::to_value(&b).expect("serialize");
    assert_eq!(json["status"], "pending");
    assert_eq!(json["read"], false);
    assert_eq!(json["title"], serde_json::Value::Null);
}
