//! Domain model — the read-later "bookmark".
//!
//! Step 1 lives here. A Java `enum` *with a getter* becomes a
//! fieldless Rust enum whose per-variant data lives in a `match`, and a POJO /
//! `record` becomes a `struct` with `pub` fields.

use serde::{Deserialize, Serialize};

/// Lifecycle of a saved bookmark.
///
/// A brand-new bookmark is `Pending` (not yet fetched); the background worker
/// flips it to `Ready` (title + excerpt extracted) or `Failed` (fetch error).
///
/// Java/Spring:
/// ```java
/// enum BookmarkStatus { PENDING, READY, FAILED;
///   String value() { return name().toLowerCase(); } }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BookmarkStatus {
    Pending,
    Ready,
    Failed,
}

impl BookmarkStatus {
    /// The lowercase string persisted in the SQLite `status` column.
    ///
    /// Java: `name().toLowerCase()`. Here: one `match self { ... }` arm per
    /// variant returning a `&'static str` (`"pending" | "ready" | "failed"`).
    pub fn as_str(&self) -> &'static str {
        todo!("step 1: map each BookmarkStatus variant to its lowercase &'static str")
    }
}

/// A saved bookmark.
///
/// Java/Spring: a `@Entity`/`record` with these fields. `Option<T>` ≈
/// `Optional<T>` — `title`/`excerpt`/`error` are absent until enrichment runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub excerpt: Option<String>,
    pub status: BookmarkStatus,
    pub error: Option<String>,
    pub read: bool,
    pub tags: Vec<String>,
    /// Creation time as Unix milliseconds (kept simple — no chrono/time dep).
    pub created_at: i64,
}

impl Bookmark {
    /// Build a freshly-saved bookmark. New bookmarks ALWAYS start `Pending`,
    /// with no `title`/`excerpt`/`error` and `read == false`.
    ///
    /// Java/Spring:
    /// ```java
    /// var b = new Bookmark(id, url, tags, createdAt);
    /// b.setStatus(PENDING); b.setRead(false);
    /// ```
    pub fn new(id: String, url: String, tags: Vec<String>, created_at: i64) -> Self {
        todo!("step 1: construct a Pending Bookmark; title/excerpt/error = None, read = false")
    }
}
