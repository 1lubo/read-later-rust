//! In-memory `AsyncBookmarkStore` — the test backend.
//!
//! Step 2 lives here. Pure data-structure work over a `HashMap` behind an
//! `Arc<Mutex<..>>` (a shared, cloneable handle).
//! Get comfortable with ownership, `Option`, and iterator filters — no SQL yet.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::AppError;
use crate::model::BookmarkStatus;
use crate::model::Bookmark;
use crate::store::{AsyncBookmarkStore, ListQuery};

/// A `Clone`able handle over shared state. Cloning shares the same map.
///
/// Java/Spring: a singleton `@Repository` backed by a `ConcurrentHashMap`.
#[derive(Clone, Default)]
pub struct InMemoryBookmarkStore {
    inner: Arc<Mutex<HashMap<String, Bookmark>>>,
}

impl InMemoryBookmarkStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Small helper: lock the mutex, mapping a poisoned lock to a storage error.
    /// (Provided.) Use `self.guard()?` inside your methods to get the map.
    fn guard(&self) -> Result<std::sync::MutexGuard<'_, HashMap<String, Bookmark>>, AppError> {
        self.inner
            .lock()
            .map_err(|_| AppError::Storage("mutex poisoned".to_string()))
    }
}

impl AsyncBookmarkStore for InMemoryBookmarkStore {
    /// Java: `map.put(b.getId(), b)`.
    async fn insert(&self, bookmark: Bookmark) -> Result<(), AppError> {
        let _ = &self.inner;
        todo!("step 2: lock the map (self.guard()?) and insert bookmark keyed by id")
    }

    /// Java: `Optional.ofNullable(map.get(id))`.
    async fn get(&self, id: &str) -> Result<Option<Bookmark>, AppError> {
        todo!("step 2: return a clone of the bookmark with this id, if present")
    }

    /// Newest first (`created_at` desc). Apply the `read`, `tag`, and `q`
    /// filters from `ListQuery` (all optional, AND-ed). `q` is a case-insensitive
    /// substring over title + excerpt.
    ///
    /// Java: `map.values().stream().filter(...).sorted(...).toList()`.
    async fn list(&self, query: &ListQuery) -> Result<Vec<Bookmark>, AppError> {
        let _ = query;
        todo!("step 2: collect, filter by read/tag/q, sort by created_at desc")
    }

    /// Return `false` if the id was absent. Java: `if (!map.containsKey(id)) ...`.
    async fn set_read(&self, id: &str, read: bool) -> Result<bool, AppError> {
        let _ = (id, read);
        todo!("step 2: set the read flag; return whether the id existed")
    }

    /// Enrichment success: set title/excerpt and status = Ready, clear error.
    async fn mark_ready(
        &self,
        id: &str,
        title: Option<String>,
        excerpt: Option<String>,
    ) -> Result<(), AppError> {
        let _ = (id, title, excerpt, BookmarkStatus::Ready);
        todo!("step 2: update the entry's title/excerpt, set status = Ready, error = None")
    }

    /// Enrichment failure: set status = Failed and record the error string.
    async fn mark_failed(&self, id: &str, error: String) -> Result<(), AppError> {
        let _ = (id, error);
        todo!("step 2: set the entry's status = Failed and error = Some(error)")
    }

    /// Return `false` if the id was absent.
    async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let _ = id;
        todo!("step 2: remove the id; return whether it existed")
    }
}
