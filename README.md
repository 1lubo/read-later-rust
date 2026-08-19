# read-later-rust

A test-driven Rust course for Java/Spring developers — the follow-on to
`fragments-api-rust`. You build a **personal read-later / bookmark service** you
can deploy to Fly.io and actually use, and along the way you meet the Rust that
Fragments didn't cover: outbound HTTP, HTML parsing, server-side templating,
self-written auth middleware, and SQLite + full-text search.

The design lives in [`docs/plans/2026-08-19-read-later-rust-design.md`](docs/plans/2026-08-19-read-later-rust-design.md).

## The loop

Same rhythm as Fragments — one red test at a time:

1. Run the active step's tests and read the failure.
2. Implement the `todo!()` stub(s) it points at (each carries a Java/Spring breadcrumb).
3. `cargo test` until green.
4. Delete the next step's `#[ignore]` line(s) to unlock it.
5. Repeat.

On a fresh clone, **only step 1 runs**; steps 2–10 are gated with
`#[ignore = "step N: ..."]`. Everything is **offline and Docker-free** — the
whole suite uses an in-memory store and temp-file SQLite, and a `FakeFetcher`
stands in for the network.

```bash
cargo test                              # runs step 1 only (fails on todo!())
cargo test --test step_02_in_memory -- --ignored   # unlock/run a later step
```

## What you're building

Save a URL → it's stored `Pending` → a background worker fetches the page,
extracts a title + excerpt, and flips it to `Ready` (or `Failed`). List / filter
/ search / tag / mark-read / delete, via a JSON API **and** a thin
server-rendered web UI.

## Steps

| # | Focus | File(s) | What's new vs. Fragments |
|---|-------|---------|--------------------------|
| 1 | Domain model | `src/model.rs` | (warm-up: enum `as_str`, constructor) |
| 2 | In-memory store | `src/in_memory.rs` | async `AsyncBookmarkStore` trait |
| 3 | SQLite store | `src/sqlite.rs` | `sqlx` CRUD + tag join + migrations |
| 4 | Full-text search | `src/sqlite.rs` | FTS5 index kept in sync from Rust |
| 5 | Page fetcher | `src/fetcher.rs` | `scraper` HTML parsing (`extract_meta`) |
| 6 | Background worker | `src/worker.rs` | `mpsc` channel + enrichment task |
| 7 | Auth | `src/auth.rs` | Bearer / signed-cookie extractor, `subtle` |
| 8 | JSON API | `src/api.rs` | handlers, filters, `tower::oneshot` tests |
| 9 | Web UI | `src/web.rs` | `askama` templates, forms, PRG, login cookie |
| 10 | Wire `main` | `src/config.rs`, `src/main.rs` | `Config::from_env`, `cargo run` |

## Rust ↔ Java/Spring cheat-sheet

| Rust | Java/Spring | Note |
|------|-------------|------|
| `trait` + enum `StoreHandle` | repository interface + Strategy impls | the storage seam |
| `Option<T>` / `Result<T, E>` + `?` | `Optional` / checked exceptions | absence & errors |
| `Arc<Mutex<T>>` | singleton bean | shared, thread-safe state |
| `tokio::mpsc` + `Worker::run` | a work queue + `@Async` listener | background enrichment |
| `axum::Router` + extractors | `@RestController` / `@Controller` | routing & arg binding |
| `IntoResponse` | `ResponseEntity` / `@ControllerAdvice` | HTTP mapping |
| `#[derive(Template)]` (askama) | typed Thymeleaf | compile-checked HTML |
| `sqlx::query(..).bind(..)` | `JdbcTemplate` w/ params | never string-format SQL |

## Layout

```
src/            model, error, store, in_memory, sqlite, fetcher, worker,
                auth, api, web, state, config, main
templates/      askama HTML (list.html, login.html)
migrations/     sqlx SQLite migrations (bookmarks, tags, join, FTS5)
tests/          step_01..step_10 + common/ harness (MockMvc-style)
Dockerfile      multi-stage release build
fly.toml        Fly.io deploy (SQLite on a volume)
```

## Deploy (final milestone)

```bash
fly launch --no-deploy          # then set app name in fly.toml
fly volumes create data --size 1 --region lhr
fly secrets set BOOKMARK_TOKEN=... COOKIE_KEY=...   # COOKIE_KEY >= 64 bytes
fly deploy
```

`COOKIE_KEY` must be at least 64 bytes. Locally, export `BOOKMARK_TOKEN` and
`COOKIE_KEY` (and optionally `DATABASE_URL` / `BIND_ADDR`) before `cargo run`.

## Bonus

The async store seam is already in place, so a natural stretch goal — mirroring
Fragments' Milestone B — is a **Postgres backend** behind the same
`AsyncBookmarkStore` trait, feature-gated alongside SQLite.
