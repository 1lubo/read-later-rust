//! Step 9 — server-rendered web UI (`src/web.rs`) via `tower::oneshot`.
//!
//! Cookie auth + Post/Redirect/Get. We log in, capture the signed `session`
//! cookie from `set-cookie`, then replay it on subsequent requests. Offline.
//! Unlock with `cargo test --test step_09_web -- --ignored`.
//!
//! Java/Spring: a `MockMvc` flow that posts a login form, grabs the session
//! cookie, and reuses it on a secured GET.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{body_string, send, test_app, TOKEN};

/// Log in and return the raw `set-cookie` header value (the signed session).
async fn login(app: &common::TestApp) -> String {
    let req = Request::post("/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(format!("token={TOKEN}")))
        .unwrap();
    let resp = send(app.router.clone(), req).await;
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    resp.headers()
        .get("set-cookie")
        .expect("a session cookie was set")
        .to_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
#[ignore = "step 9: unauthenticated GET / redirects to /login"]
async fn index_redirects_when_logged_out() {
    let app = test_app();
    let req = Request::get("/").body(Body::empty()).unwrap();
    let resp = send(app.router.clone(), req).await;
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    assert_eq!(resp.headers().get("location").unwrap(), "/login");
}

#[tokio::test]
#[ignore = "step 9: wrong token re-renders the login page (not a redirect)"]
async fn login_with_wrong_token_rerenders() {
    let app = test_app();
    let req = Request::post("/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("token=nope"))
        .unwrap();
    let resp = send(app.router.clone(), req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(body_string(resp).await.to_lowercase().contains("token"));
}

#[tokio::test]
#[ignore = "step 9: login sets a session cookie that authorizes GET /"]
async fn login_then_index_renders_list() {
    let app = test_app();
    let cookie = login(&app).await;

    let req = Request::get("/")
        .header("cookie", cookie)
        .body(Body::empty())
        .unwrap();
    let resp = send(app.router.clone(), req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let html = body_string(resp).await;
    assert!(html.contains("read-later"));
}

#[tokio::test]
#[ignore = "step 9: POST / adds a bookmark and redirects (PRG)"]
async fn add_form_saves_and_redirects() {
    let app = test_app();
    let cookie = login(&app).await;

    let req = Request::post("/")
        .header("cookie", cookie)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("url=https://example.com&tags=rust, web"))
        .unwrap();
    let resp = send(app.router.clone(), req).await;
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    assert_eq!(resp.headers().get("location").unwrap(), "/");

    use readlater::store::{AsyncBookmarkStore, ListQuery};
    let all = app.store.list(&ListQuery::default()).await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].url, "https://example.com");
}
