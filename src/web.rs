//! Server-rendered web UI (step 9).
//!
//! Java/Spring: `web.rs` is an `@Controller` (not `@RestController`) returning
//! rendered views. `askama` templates are compile-time-checked HTML — the
//! typed-Thymeleaf you asked for: a `#[derive(Template)]` struct is the model
//! the view binds to, and a missing field or typo'd `{{ ... }}` is a COMPILE
//! error, not a runtime 500. Forms use Post/Redirect/Get so a refresh can't
//! double-submit. Auth here is the signed `session` cookie, not a Bearer token.

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use axum_extra::extract::SignedCookieJar;
use serde::Deserialize;

use crate::model::Bookmark;
use crate::state::AppState;

/// Model bound to `templates/list.html`: the visible bookmarks + the current
/// search box contents. Java/Spring: the `Model` attributes for the view.
#[derive(Template)]
#[template(path = "list.html")]
pub struct ListTemplate {
    pub bookmarks: Vec<Bookmark>,
    pub q: String,
}

/// Model bound to `templates/login.html`.
#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
}

/// Query string for the list page (`/?q=...`).
#[derive(Debug, Default, Deserialize)]
pub struct WebListParams {
    pub q: Option<String>,
}

/// Add-bookmark form fields (`application/x-www-form-urlencoded`).
#[derive(Debug, Deserialize)]
pub struct AddForm {
    pub url: String,
    #[serde(default)]
    pub tags: String,
}

/// Login form field.
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub token: String,
}

/// Render a template to an HTML response. (Provided helper.)
fn render<T: Template>(tpl: T) -> Response {
    match tpl.render() {
        Ok(body) => Html(body).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "template render failed");
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "render error").into_response()
        }
    }
}

/// `GET /`: require the session cookie; list bookmarks (honoring `?q=`); render
/// `list.html`. If not logged in, redirect to `/login`.
///
/// Breadcrumbs: check the signed `session` cookie via `jar.get("session")` +
/// `state.auth.token_matches`; on miss `Redirect::to("/login").into_response()`.
async fn index(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Query(params): Query<WebListParams>,
) -> Response {
    let _ = (&state, &jar, params);
    todo!("step 9: auth via session cookie; list(q) -> render(ListTemplate) else redirect /login")
}

/// `POST /`: add a bookmark from the form (split `tags` on commas), enqueue it,
/// then Post/Redirect/Get back to `/`.
async fn add(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Form(form): Form<AddForm>,
) -> Response {
    let _ = (&state, &jar, form);
    todo!("step 9: auth; validate+insert Pending; enqueue; Redirect::to(\"/\")")
}

/// `POST /{id}/read`: toggle-to-read, then redirect to `/`.
async fn mark_read(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(id): Path<String>,
) -> Response {
    let _ = (&state, &jar, id);
    todo!("step 9: auth; set_read(id, true); Redirect::to(\"/\")")
}

/// `POST /{id}/delete`: delete, then redirect to `/`.
async fn delete(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(id): Path<String>,
) -> Response {
    let _ = (&state, &jar, id);
    todo!("step 9: auth; delete(id); Redirect::to(\"/\")")
}

/// `GET /login`: render the login form (no error on first view).
async fn login_form() -> Response {
    render(LoginTemplate { error: None })
}

/// `POST /login`: if the token matches, set the signed `session` cookie and
/// redirect to `/`; otherwise re-render `login.html` with an error.
///
/// Breadcrumbs: `state.auth.token_matches(&form.token)`; on success build a
/// `Cookie::new("session", token)` (consider `.http_only(true)`, `.path("/")`),
/// `jar.add(cookie)`, and return `(updated_jar, Redirect::to("/"))`.
async fn login_submit(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Form(form): Form<LoginForm>,
) -> Response {
    let _ = (&state, jar, form);
    todo!("step 9: verify token -> set signed session cookie + redirect, else re-render with error")
}

/// Assemble the web router. (Provided.)
pub fn web_router() -> Router<AppState> {
    Router::new()
        .route("/", get(index).post(add))
        .route("/{id}/read", post(mark_read))
        .route("/{id}/delete", post(delete))
        .route("/login", get(login_form).post(login_submit))
}
