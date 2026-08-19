//! The storage seam.
//!
//! `AsyncBookmarkStore` ≈ a Spring `BookmarkRepository` interface. Two backends
//! implement it: `InMemoryBookmarkStore` (tests) and `SqliteBookmarkStore`
//! (production). `StoreHandle` is the concrete enum wired into the app — it
//! dispatches to the chosen backend (an enum avoids `dyn` with
//! async-fn-in-trait). The delegation below is provided plumbing.

use crate::error::AppError;
use crate::in_memory::InMemoryBookmarkStore;
use crate::model::Bookmark;
use crate::sqlite::SqliteBookmarkStore;

/// Filters for `list`. All optional and they compose (AND-ed together).
///
/// Java/Spring: the `@RequestParam` bundle on `GET /bookmarks`.
#[derive(Debug, Default, Clone)]
pub struct ListQuery {
    /// `Some(true)` → only read, `Some(false)` → only unread, `None` → all.
    pub read: Option<bool>,
    /// Filter to bookmarks carrying this tag.
    pub tag: Option<String>,
    /// Full-text query across title + excerpt.
    pub q: Option<String>,
}

/// The repository seam. `&self` methods only — backends carry their own shared,
/// cloneable handle (an `Arc<Mutex<..>>` or a connection pool).
#[allow(async_fn_in_trait)]
pub trait AsyncBookmarkStore {
    async fn insert(&self, bookmark: Bookmark) -> Result<(), AppError>;
    async fn get(&self, id: &str) -> Result<Option<Bookmark>, AppError>;
    async fn list(&self, query: &ListQuery) -> Result<Vec<Bookmark>, AppError>;
    /// Set the read flag. Returns `false` if no such id existed.
    async fn set_read(&self, id: &str, read: bool) -> Result<bool, AppError>;
    /// Enrichment success: store title/excerpt and flip status to `Ready`.
    async fn mark_ready(
        &self,
        id: &str,
        title: Option<String>,
        excerpt: Option<String>,
    ) -> Result<(), AppError>;
    /// Enrichment failure: store the error and flip status to `Failed`.
    async fn mark_failed(&self, id: &str, error: String) -> Result<(), AppError>;
    /// Delete. Returns `false` if no such id existed.
    async fn delete(&self, id: &str) -> Result<bool, AppError>;
}

/// The backend actually wired into `AppState`. Provided.
#[derive(Clone)]
pub enum StoreHandle {
    Memory(InMemoryBookmarkStore),
    Sqlite(SqliteBookmarkStore),
}

impl AsyncBookmarkStore for StoreHandle {
    async fn insert(&self, bookmark: Bookmark) -> Result<(), AppError> {
        match self {
            StoreHandle::Memory(s) => s.insert(bookmark).await,
            StoreHandle::Sqlite(s) => s.insert(bookmark).await,
        }
    }
    async fn get(&self, id: &str) -> Result<Option<Bookmark>, AppError> {
        match self {
            StoreHandle::Memory(s) => s.get(id).await,
            StoreHandle::Sqlite(s) => s.get(id).await,
        }
    }
    async fn list(&self, query: &ListQuery) -> Result<Vec<Bookmark>, AppError> {
        match self {
            StoreHandle::Memory(s) => s.list(query).await,
            StoreHandle::Sqlite(s) => s.list(query).await,
        }
    }
    async fn set_read(&self, id: &str, read: bool) -> Result<bool, AppError> {
        match self {
            StoreHandle::Memory(s) => s.set_read(id, read).await,
            StoreHandle::Sqlite(s) => s.set_read(id, read).await,
        }
    }
    async fn mark_ready(
        &self,
        id: &str,
        title: Option<String>,
        excerpt: Option<String>,
    ) -> Result<(), AppError> {
        match self {
            StoreHandle::Memory(s) => s.mark_ready(id, title, excerpt).await,
            StoreHandle::Sqlite(s) => s.mark_ready(id, title, excerpt).await,
        }
    }
    async fn mark_failed(&self, id: &str, error: String) -> Result<(), AppError> {
        match self {
            StoreHandle::Memory(s) => s.mark_failed(id, error).await,
            StoreHandle::Sqlite(s) => s.mark_failed(id, error).await,
        }
    }
    async fn delete(&self, id: &str) -> Result<bool, AppError> {
        match self {
            StoreHandle::Memory(s) => s.delete(id).await,
            StoreHandle::Sqlite(s) => s.delete(id).await,
        }
    }
}
