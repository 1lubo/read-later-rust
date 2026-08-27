//! JSON API handlers (step 8).
//!
//! Java/Spring: `api.rs` is a `@RestController`. Each function is a
//! `@GetMapping`/`@PostMapping`/... method; the `State` extractor is
//! constructor-injected dependencies; `Json<T>` in an argument is
//! `@RequestBody`, and `Json<T>` returned is the serialized body. The presence
//! of `AuthToken` in the argument list is `@PreAuthorize` — the route is gated.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use crate::auth::AuthToken;
use crate::error::AppError;
use crate::state::AppState;
use crate::store::ListQuery;

/// Request body for `POST /bookmarks`. Java/Spring: a `record CreateBookmark`.
#[derive(Debug, Deserialize)]
pub struct CreateBookmark {
    pub url: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Body for `PATCH /bookmarks/{id}`.
#[derive(Debug, Deserialize)]
pub struct UpdateBookmark {
    pub read: bool,
}

/// Raw `?read=&tag=&q=` query params, later mapped into a `ListQuery`.
#[derive(Debug, Default, Deserialize)]
pub struct ListParams {
    pub read: Option<bool>,
    pub tag: Option<String>,
    pub q: Option<String>,
}

impl ListParams {
    /// Provided helper: fold the raw params into the store's `ListQuery`.
    pub fn into_query(self) -> ListQuery {
        ListQuery {
            read: self.read,
            tag: self.tag,
            q: self.q,
        }
    }
}

/// Liveness probe. The ONLY unauthenticated route. (Provided.)
async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

/// `POST /bookmarks`: validate the URL, build a `Pending` bookmark, insert it,
/// enqueue its id for enrichment, and return `201 Created` + the bookmark JSON.
///
/// Breadcrumbs: reject with `AppError::InvalidUrl` unless the url starts with
/// `http://`/`https://`; mint an id with `uuid::Uuid::new_v4()`; timestamp with
/// the provided `now_millis()`; `state.enqueue.send(id).await` (ignore a send
/// error for now); finish with `(StatusCode::CREATED, Json(bookmark)).into_response()`.
///
/// The return type is the concrete `Response` (not `impl IntoResponse`): a bare
/// `todo!()` body gives the compiler no type to infer for `impl Trait`, so you
/// call `.into_response()` on your final tuple. Java/Spring: like declaring the
/// method returns `ResponseEntity<?>` instead of a raw domain object.
async fn create_bookmark(
    State(state): State<AppState>,
    _auth: AuthToken,
    Json(body): Json<CreateBookmark>,
) -> Result<Response, AppError> {
    let _ = (&state, body);
    todo!("step 8: validate url -> insert Pending -> enqueue id -> 201 + Json(bookmark)")
}

/// `GET /bookmarks`: map params to `ListQuery`, return `Json(Vec<Bookmark>)`.
async fn list_bookmarks(
    State(state): State<AppState>,
    _auth: AuthToken,
    Query(params): Query<ListParams>,
) -> Result<Response, AppError> {
    let _ = (&state, params);
    todo!("step 8: state.store.list(&params.into_query()).await -> Json(list).into_response()")
}

/// `GET /bookmarks/{id}`: `200` + Json, or `404` (`AppError::NotFound`).
async fn get_bookmark(
    State(state): State<AppState>,
    _auth: AuthToken,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let _ = (&state, id);
    todo!("step 8: get(id) -> Json(bookmark).into_response() or AppError::NotFound")
}

/// `PATCH /bookmarks/{id}`: set the read flag; `404` if the id is unknown.
async fn update_bookmark(
    State(state): State<AppState>,
    _auth: AuthToken,
    Path(id): Path<String>,
    Json(body): Json<UpdateBookmark>,
) -> Result<Response, AppError> {
    let _ = (&state, id, body);
    todo!("step 8: set_read(id, body.read); false -> NotFound; then return updated Json")
}

/// `DELETE /bookmarks/{id}`: `204` on success, `404` if unknown.
async fn delete_bookmark(
    State(state): State<AppState>,
    _auth: AuthToken,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let _ = (&state, id);
    todo!("step 8: delete(id); true -> StatusCode::NO_CONTENT, false -> NotFound")
}

/// Assemble the JSON router. Java/Spring: the `@RequestMapping` wiring implied
/// by the annotations, made explicit. (Provided — mirrors the handlers above.)
pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/bookmarks", post(create_bookmark).get(list_bookmarks))
        .route(
            "/bookmarks/{id}",
            get(get_bookmark)
                .patch(update_bookmark)
                .delete(delete_bookmark),
        )
}

/// Current time in Unix milliseconds. (Provided helper.)
pub fn now_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
