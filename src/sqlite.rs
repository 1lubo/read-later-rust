//! SQLite `AsyncBookmarkStore` — the production backend (steps 3 & 4).
//!
//! Step 3: `connect` / `migrate` + type-safe CRUD and the tags join table.
//! Step 4: full-text search (`?q=`) via the `bookmarks_fts` FTS5 table, kept in
//! sync from Rust on insert / mark_ready / delete.
//!
//! SQL rule (as in Fragments): build queries with the RUNTIME api
//! (`sqlx::query(...)` / `sqlx::query_as(...)`), never the compile-time-checked
//! `query!` macros — so the crate compiles with no live database. Bind every
//! value with `.bind(..)`; never string-format user input into SQL.

use sqlx::sqlite::SqlitePool;

use crate::error::AppError;
use crate::model::Bookmark;
use crate::store::{AsyncBookmarkStore, ListQuery};

/// A pooled SQLite backend. `SqlitePool` is cheap to `Clone` (shared pool).
///
/// Java/Spring: the `DataSource` / HikariCP-backed `JdbcTemplate`.
#[derive(Clone)]
pub struct SqliteBookmarkStore {
    pool: SqlitePool,
}

impl SqliteBookmarkStore {
    /// Open a pool from a URL such as `sqlite:/data/bookmarks.db?mode=rwc` or
    /// `sqlite::memory:`.
    ///
    /// Breadcrumb: `sqlx::sqlite::SqlitePoolOptions::new().connect(database_url).await`,
    /// then wrap the pool in `Self`.
    pub async fn connect(database_url: &str) -> Result<Self, AppError> {
        let _ = database_url;
        todo!("step 3: build a SqlitePool from database_url and wrap it in Self")
    }

    /// Run migrations on boot. Java/Spring: Flyway/Liquibase.
    ///
    /// Breadcrumb: `sqlx::migrate!(\"./migrations\").run(&self.pool).await`.
    pub async fn migrate(&self) -> Result<(), AppError> {
        todo!("step 3: run the embedded migrations against self.pool")
    }

    /// Escape hatch for tests / wiring that need the raw pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

impl AsyncBookmarkStore for SqliteBookmarkStore {
    /// INSERT the row, then upsert each tag into `tags` and link it in
    /// `bookmark_tags`. Also seed the `bookmarks_fts` row (title/excerpt may be
    /// NULL for now).
    async fn insert(&self, bookmark: Bookmark) -> Result<(), AppError> {
        let _ = (&self.pool, bookmark);
        todo!("step 3: INSERT the bookmark row + upsert/link its tags")
    }

    /// SELECT one row by id and aggregate its tags (e.g. a second query or a
    /// `GROUP_CONCAT` join). Return `None` if absent.
    async fn get(&self, id: &str) -> Result<Option<Bookmark>, AppError> {
        let _ = id;
        todo!("step 3: SELECT the row by id and load its tags; map to Bookmark")
    }

    /// List newest-first with optional filters. Step 3: `read` + `tag`.
    /// Step 4: when `query.q` is set, join `bookmarks_fts` with
    /// `WHERE bookmarks_fts MATCH ?`.
    async fn list(&self, query: &ListQuery) -> Result<Vec<Bookmark>, AppError> {
        let _ = query;
        todo!("step 3: read/tag filters, ORDER BY created_at DESC; step 4: add FTS MATCH for q")
    }

    async fn set_read(&self, id: &str, read: bool) -> Result<bool, AppError> {
        let _ = (id, read);
        todo!("step 3: UPDATE read; return whether a row was affected")
    }

    /// Step 3: UPDATE title/excerpt + status='ready', clear error.
    /// Step 4: keep the matching `bookmarks_fts` row in sync.
    async fn mark_ready(
        &self,
        id: &str,
        title: Option<String>,
        excerpt: Option<String>,
    ) -> Result<(), AppError> {
        let _ = (id, title, excerpt);
        todo!("step 3: UPDATE title/excerpt/status=ready; step 4: sync FTS row")
    }

    async fn mark_failed(&self, id: &str, error: String) -> Result<(), AppError> {
        let _ = (id, error);
        todo!("step 3: UPDATE status='failed' and error")
    }

    /// DELETE the row (the `ON DELETE CASCADE` FK clears `bookmark_tags`); also
    /// remove its `bookmarks_fts` row. Return whether a row existed.
    async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let _ = id;
        todo!("step 3: DELETE the bookmark (+ its FTS row); return whether it existed")
    }
}
