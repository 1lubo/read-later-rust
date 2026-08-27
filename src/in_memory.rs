//! In-memory `AsyncBookmarkStore` — the test backend.
//!
//! Step 2 lives here. Pure data-structure work over a `HashMap` behind an
//! `Arc<Mutex<..>>` (a shared, cloneable handle).
//! Get comfortable with ownership, `Option`, and iterator filters — no SQL yet.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::AppError;
use crate::model::Bookmark;
use crate::model::BookmarkStatus;
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
        let id: String = bookmark.id.clone();
        let bookmark_clone = bookmark.clone();
        self.guard()?.insert(id, bookmark_clone);
        Ok(())
    }

    /// Java: `Optional.ofNullable(map.get(id))`.
    async fn get(&self, id: &str) -> Result<Option<Bookmark>, AppError> {
        match self.guard()?.get(id) {
            Some(bookmark) => Ok(Some(bookmark.clone())),
            None => Ok(None),
        }
    }

    /// Newest first (`created_at` desc). Apply the `read`, `tag`, and `q`
    /// filters from `ListQuery` (all optional, AND-ed). `q` is a case-insensitive
    /// substring over title + excerpt.
    ///
    /// Java: `map.values().stream().filter(...).sorted(...).toList()`.
    async fn list(&self, query: &ListQuery) -> Result<Vec<Bookmark>, AppError> {
        let map = self.guard()?;
        let mut out: Vec<Bookmark> = map
            .values()
            .filter(|b| query.read.map_or(true, |want| b.read == want))
            .filter(|b| query.tag.as_ref().map_or(true, |t| b.tags.contains(t)))
            .filter(|b| {
                query.q.as_ref().map_or(true, |needle| {
                    let needle = needle.to_lowercase();
                    b.title
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&needle)
                        || b.excerpt
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&needle)
                })
            })
            .cloned()
            .collect();
        out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(out)
    }

    /// Return `false` if the id was absent. Java: `if (!map.containsKey(id)) ...`.
    async fn set_read(&self, id: &str, read: bool) -> Result<bool, AppError> {
        let mut map = self.guard()?;
        Ok(map.get_mut(id).map(|b| b.read = read).is_some())
    }

    /// Enrichment success: set title/excerpt and status = Ready, clear error.
    async fn mark_ready(
        &self,
        id: &str,
        title: Option<String>,
        excerpt: Option<String>,
    ) -> Result<(), AppError> {
        let mut map = self.guard()?;
        if let Some(b) = map.get_mut(id) {
            b.title = title;
            b.excerpt = excerpt;
            b.status = BookmarkStatus::Ready;
            b.error = None;
        }
        Ok(())
    }

    /// Enrichment failure: set status = Failed and record the error string.
    async fn mark_failed(&self, id: &str, error: String) -> Result<(), AppError> {
        let mut map = self.guard()?;
        if let Some(b) = map.get_mut(id) {
            b.status = BookmarkStatus::Failed;
            b.error = Some(error);
        }
        Ok(())
    }

    /// Return `false` if the id was absent.
    async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let mut map = self.guard()?;
        Ok(map.remove(id).is_some())
    }
}
