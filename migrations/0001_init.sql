-- Initial schema for read-later (steps 3 & 4).
--
-- Java/Spring: this is your Flyway V1__init.sql. sqlx::migrate!("./migrations")
-- embeds every *.sql here at compile time and applies the unapplied ones on
-- boot, tracking them in a _sqlx_migrations table (like flyway_schema_history).

PRAGMA foreign_keys = ON;

-- One row per saved bookmark. status is a lowercase string matching
-- BookmarkStatus::as_str(): 'pending' | 'ready' | 'failed'.
CREATE TABLE bookmarks (
    id         TEXT PRIMARY KEY,
    url        TEXT NOT NULL,
    title      TEXT,
    excerpt    TEXT,
    status     TEXT NOT NULL,
    error      TEXT,
    read       INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_bookmarks_created_at ON bookmarks (created_at DESC);

-- Tags, and the many-to-many join to bookmarks.
CREATE TABLE tags (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE bookmark_tags (
    bookmark_id TEXT NOT NULL REFERENCES bookmarks (id) ON DELETE CASCADE,
    tag_id      INTEGER NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    PRIMARY KEY (bookmark_id, tag_id)
);

CREATE INDEX idx_bookmark_tags_tag ON bookmark_tags (tag_id);

-- Full-text search over title + excerpt (step 4). We keep this in sync from
-- Rust (insert / mark_ready / delete) rather than via triggers, so the sync is
-- visible and testable. content_rowid ties each FTS row to a bookmark via a
-- stable string key stored in the `bid` column.
CREATE VIRTUAL TABLE bookmarks_fts USING fts5 (
    bid UNINDEXED,
    title,
    excerpt
);
