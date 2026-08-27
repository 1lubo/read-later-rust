//! SQLite `AsyncBookmarkStore` — the production backend (steps 3 & 4).
//!
//! Step 3: `connect` / `migrate` + type-safe CRUD and the tags join table.
//! Step 4: full-text search (`?q=`) via the `bookmarks_fts` FTS5 table, kept in
//! sync from Rust on insert / mark_ready / delete.
//!
//! SQL rule: build queries with the RUNTIME api
//! (`sqlx::query(...)` / `sqlx::query_as(...)`), never the compile-time-checked
//! `query!` macros — so the crate compiles with no live database. Bind every
//! value with `.bind(..)`; never string-format user input into SQL.

use crate::error::AppError;
use crate::model::{Bookmark, BookmarkStatus};
use crate::store::{AsyncBookmarkStore, ListQuery};
use sqlx::sqlite::SqlitePool;
use sqlx::QueryBuilder;

/// A pooled SQLite backend. `SqlitePool` is cheap to `Clone` (shared pool).
///
/// Java/Spring: the `DataSource` / HikariCP-backed `JdbcTemplate`.

#[derive(sqlx::FromRow)]
struct BookmarkRow {
    id: String,
    url: String,
    title: Option<String>,
    excerpt: Option<String>,
    #[sqlx(try_from = "String")]
    status: BookmarkStatus,
    error: Option<String>,
    read: bool,
    created_at: i64,
}

impl BookmarkRow {
    fn into_bookmark(self, tags: Vec<String>) -> Bookmark {
        Bookmark {
            id: self.id, url: self.url, title: self.title, excerpt: self.excerpt, status: self.status,
            error: self.error, read: self.read, tags, created_at: self.created_at
        }
    }
}
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
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }

    /// Run migrations on boot. Java/Spring: Flyway/Liquibase.
    ///
    /// Breadcrumb: `sqlx::migrate!(\"./migrations\").run(&self.pool).await`.
    pub async fn migrate(&self) -> Result<(), AppError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| AppError::Storage(e.to_string()))
    }

    /// Escape hatch for tests / wiring that need the raw pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    async fn load_tags(&self, id: &str) -> Result<Vec<String>, AppError> {
        Ok(sqlx::query_scalar(
            "SELECT t.name FROM tags t JOIN bookmark_tags bt ON bt.tag_id = t.id
             WHERE bt.bookmark_id = ?")
            .bind(id)
            .fetch_all(&self.pool)
            .await?
        )
    }
}

impl AsyncBookmarkStore for SqliteBookmarkStore {
    /// INSERT the row, then upsert each tag into `tags` and link it in
    /// `bookmark_tags`. Also seed the `bookmarks_fts` row (title/excerpt may be
    /// NULL for now).
    async fn insert(&self, bookmark: Bookmark) -> Result<(), AppError> {
        let mut conn = self.pool.begin().await?;

        sqlx::query("INSERT INTO bookmarks(id, url, title, excerpt, status, error, read, created_at)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?)", )
            .bind(&bookmark.id)
            .bind(&bookmark.url)
            .bind(&bookmark.title)
            .bind(&bookmark.excerpt)
            .bind(&bookmark.status.as_str())
            .bind(&bookmark.error)
            .bind(&bookmark.read)
            .bind(&bookmark.created_at)
            .execute(&mut *conn)
            .await?;

        for tag in &bookmark.tags {
            sqlx::query("INSERT OR IGNORE INTO tags(name) VALUES(?)")
                .bind(&tag)
                .execute(&mut *conn)
                .await?;

            let tag_id: i64 = sqlx::query_scalar("SELECT id FROM tags WHERE name=?")
                .bind(&tag)
                .fetch_one(&mut *conn)
                .await?;

            sqlx::query("INSERT OR IGNORE INTO bookmark_tags(bookmark_id, tag_id) VALUES(?, ?)")
                .bind(&bookmark.id)
                .bind(&tag_id)
                .execute(&mut *conn)
                .await?;
        }

        sqlx::query("INSERT INTO bookmarks_fts(bid, title, excerpt) VALUES(?, ?, ?)")
            .bind(&bookmark.id)
            .bind(&bookmark.title)
            .bind(&bookmark.excerpt)
            .execute(&mut *conn)
            .await?;

        conn.commit().await?;
        Ok(())
    }

    /// SELECT one row by id and aggregate its tags (e.g. a second query or a
    /// `GROUP_CONCAT` join). Return `None` if absent.
    async fn get(&self, id: &str) -> Result<Option<Bookmark>, AppError> {
        let row = sqlx::query_as::<_, BookmarkRow>("SELECT * FROM bookmarks WHERE id=?")
            .bind(&id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let tags = self.load_tags(id).await?;
                Ok(Some(row.into_bookmark(tags)))
            }
            None => Ok(None),
        }
    }

    /// List newest-first with optional filters. Step 3: `read` + `tag`.
    /// Step 4: when `query.q` is set, join `bookmarks_fts` with
    /// `WHERE bookmarks_fts MATCH ?`.
    async fn list(&self, query: &ListQuery) -> Result<Vec<Bookmark>, AppError> {
        let mut qb = QueryBuilder::new("SELECT b.* FROM bookmarks b");

        if let Some(query) = &query.q {
            qb.push(" JOIN bookmarks_fts ON bookmarks_fts.bid = b.id");
            qb.push(" WHERE bookmarks_fts MATCH ").push_bind(query);
        } else {
            qb.push(" WHERE 1 = 1");
        }

        if let Some(read) = &query.read {
            qb.push(" AND b.read = ").push_bind(read);
        }

        if let Some(tag) = &query.tag {
            qb.push(" AND b.id IN (SELECT bt.bookmark_id FROM bookmark_tags bt
            JOIN tags t ON t.id = bt.tag_id WHERE t.name = ")
                .push_bind(tag).push(")");
        }

        qb.push(" ORDER BY b.created_at DESC");

        let rows = qb.build_query_as::<BookmarkRow>().fetch_all(&self.pool).await?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let tags = self.load_tags(&row.id).await?;
            out.push(row.into_bookmark(tags));
        }
        Ok(out)
    }

    async fn set_read(&self, id: &str, read: bool) -> Result<bool, AppError> {
        let result = sqlx::query("UPDATE bookmarks SET read = ? WHERE id = ?")
            .bind(read)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Step 3: UPDATE title/excerpt + status='ready', clear error.
    /// Step 4: keep the matching `bookmarks_fts` row in sync.
    async fn mark_ready(
        &self,
        id: &str,
        title: Option<String>,
        excerpt: Option<String>,
    ) -> Result<(), AppError> {
        let mut conn = self.pool.begin().await?;
        sqlx::query("UPDATE bookmarks SET title = ?, excerpt = ?, status = ?, error = ? WHERE id = ?")
            .bind(title.as_deref())
            .bind(excerpt.as_deref())
            .bind(BookmarkStatus::Ready.as_str())
            .bind(None::<String>)
            .bind(id)
            .execute(&mut *conn)
            .await?;


        sqlx::query("UPDATE bookmarks_fts SET title = ?, excerpt = ? WHERE bid = ?")
            .bind(title.as_deref())
            .bind(excerpt.as_deref())
            .bind(id)
            .execute(&mut *conn)
            .await?;

        conn.commit().await?;
        Ok(())
    }

    async fn mark_failed(&self, id: &str, error: String) -> Result<(), AppError> {
        sqlx::query("UPDATE bookmarks SET status = ?, error = ? WHERE id = ?")
            .bind(BookmarkStatus::Failed.as_str())
            .bind(error)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// DELETE the row (the `ON DELETE CASCADE` FK clears `bookmark_tags`); also
    /// remove its `bookmarks_fts` row. Return whether a row existed.
    async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let mut conn = self.pool.begin().await?;
        let result = sqlx::query("DELETE FROM bookmarks WHERE id = ?")
            .bind(id)
            .execute(&mut *conn)
            .await?;

        let existed = result.rows_affected() > 0;


        sqlx::query("DELETE FROM bookmarks_fts WHERE bid = ?")
            .bind(id)
            .execute(&mut *conn)
            .await?;

        conn.commit().await?;
        Ok(existed)
    }
}
