# read-later-rust — design

A test-driven Rust course, milestone-staged like `fragments-api-rust`, that builds a
**personal read-later / bookmark service** you deploy to Fly.io and actually use.

## Goal

Learn the Rust you *didn't* hit in Fragments — outbound HTTP, HTML parsing, server-side
templating, self-written auth middleware, SQLite + FTS5 — by building one real service.
Same loop: read the failing test → implement the `todo!()` stub → `cargo test` → green →
delete the next `#[ignore]` line → repeat. Java/Spring breadcrumbs above every stub.

## What you're building

Save a URL; the server stores it `Pending` and a background worker fetches the page to
extract a title + excerpt, flipping it to `Ready` (or `Failed`). List / filter / search /
tag / mark-read / delete, via a JSON API and a thin server-rendered web UI.

## Decisions (confirmed)

| Area          | Choice                                                         |
|---------------|----------------------------------------------------------------|
| Interaction   | JSON API core + thin `askama` server-rendered UI               |
| Storage       | SQLite on a Fly volume (sqlx); temp-file SQLite in tests       |
| Auth          | Single static token: `Bearer` header **or** signed cookie      |
| Enrichment    | Background worker (mpsc), from the start                        |
| Search        | SQLite FTS5 over title + excerpt                               |
| Tags          | Many-to-many join table                                        |
| Time          | `created_at` unix millis (`i64`), no chrono/time dep            |
| Deploy        | Fly.io, multi-stage Docker, secrets for token + cookie key     |

## Architecture

Two trait seams carry dependency inversion (as in Fragments' `Dispatcher`/`FragmentStore`):

- `AsyncBookmarkStore` — `InMemoryBookmarkStore` (tests) vs `SqliteBookmarkStore`; a
  `StoreHandle` enum dispatches (avoids `dyn` with async-fn-in-trait).
- `PageFetcher` — `FakeFetcher` (tests, canned/offline) vs `ReqwestFetcher` (real).

Both keep the whole suite **offline and Docker-free**.

```
src/
  model.rs      Bookmark + BookmarkStatus (Pending|Ready|Failed) + serde
  error.rs      AppError + IntoResponse  (≈ @ControllerAdvice)
  store.rs      AsyncBookmarkStore trait + ListQuery + StoreHandle enum
  sqlite.rs     SqliteBookmarkStore: connect/migrate/CRUD/tags/FTS (sqlx)
  in_memory.rs  HashMap<Arc<Mutex>> behind the async store seam (test double)
  fetcher.rs    PageFetcher trait + extract_meta() + ReqwestFetcher + FakeFetcher
  worker.rs     Worker<S,F>: mpsc<id> -> fetch -> mark_ready/mark_failed
  auth.rs       AuthConfig + AuthToken extractor (bearer | signed cookie) + subtle compare
  api.rs        JSON handlers + api_router  (≈ @RestController)
  web.rs        HTML handlers + forms (askama), Post/Redirect/Get
  state.rs      AppState: StoreHandle + mpsc::Sender<String> + AuthConfig
  config.rs     Config::from_env (DATABASE_URL, BIND_ADDR, token, cookie key)
  main.rs       wire state + spawn worker + serve
templates/      askama .html (list page, add form, login)
migrations/     sqlx SQLite migrations (bookmarks, tags, bookmark_tags, FTS5)
```

## Data model

```rust
enum BookmarkStatus { Pending, Ready, Failed }   // match-based as_str()
struct Bookmark { id, url, title:Option, excerpt:Option, status,
                  error:Option, read:bool, tags:Vec<String>, created_at:i64 }
```

Schema: `bookmarks(id TEXT PK, url, title, excerpt, status, error, read INT, created_at INT)`,
`tags(id, name UNIQUE)`, `bookmark_tags(bookmark_id, tag_id)`, and `bookmarks_fts` (FTS5 over
title+excerpt, kept in sync from store code — visible + testable rather than hidden in a trigger).

## HTTP surface

JSON (all but `/healthz` require auth): `GET /healthz`, `POST /bookmarks` (201 Pending),
`GET /bookmarks?read=&tag=&q=`, `GET /bookmarks/{id}`, `PATCH /bookmarks/{id}` (read toggle),
`DELETE /bookmarks/{id}`. Web (cookie auth): `GET /` list, `POST /` add (PRG),
`POST /{id}/read`, `POST /{id}/delete`, `GET|POST /login`.

Malformed URL → `400` up front; a valid-looking URL that fails to fetch → stored `Failed`.

## Save flow

`POST /bookmarks` → validate URL → insert `Pending` → `tx.send(id)` → `201`.
Worker: `recv(id)` → load → `fetcher.fetch(url)` → `mark_ready(title,excerpt)` (+FTS) or
`mark_failed(error)`. `Worker::process(id)` is a public method so tests drive one enrichment
deterministically with `FakeFetcher`.

## Staged steps (each gated by `#[ignore = "step N: ..."]`, only step 1 active on clone)

1. model — `BookmarkStatus::as_str`, `Bookmark::new` (Pending)
2. in-memory store — async CRUD + `ListQuery` filters (read/tag)
3. SQLite store — migrate + CRUD + tag join (temp DB)
4. FTS search — `?q=` + FTS sync
5. fetcher — `extract_meta` from static HTML + `ReqwestFetcher`
6. worker — `Worker::process`: Pending → Ready/Failed (FakeFetcher)
7. auth — `token_matches` (subtle) + `AuthToken` extractor (401 paths)
8. JSON API — handlers + router, filters compose
9. web UI — askama templates, form save (PRG), read/delete, login cookie
10. wire `main` + `Config::from_env`; `cargo run`

Optional later milestones: async already baked in; **bonus** = add a Postgres backend behind
the same `AsyncBookmarkStore` seam (feature-gated), mirroring Fragments' Milestone B.

## Deploy (final milestone)

Multi-stage Dockerfile (release build → `debian:bookworm-slim` + `ca-certificates`), SQLite on
a Fly volume at `/data`, `sqlx::migrate!()` on boot. Fly secrets: `BOOKMARK_TOKEN`, `COOKIE_KEY`.
`fly launch --no-deploy` → `fly volumes create data` → `fly secrets set …` → `fly deploy`.

## Out of scope

Offline article archiving, folders/hierarchy, bulk import, pagination, multi-user, RSS, sharing.
