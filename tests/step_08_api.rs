//! Step 8 — JSON API (`src/api.rs`) end-to-end via `tower::oneshot`.
//!
//! MockMvc-style: build a `Request`, drive the merged router, assert on the
//! `Response`. Offline (in-memory store). Unlock with
//! `cargo test --test step_08_api -- --ignored`.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{bearer, body_string, send, test_app, TOKEN};
use readlater::model::BookmarkStatus;
use readlater::store::AsyncBookmarkStore;

#[tokio::test]
#[ignore = "step 8: healthz is public and returns 200"]
async fn healthz_is_public() {
    let app = test_app();
    let req = Request::get("/healthz").body(Body::empty()).unwrap();
    let resp = send(app.router.clone(), req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "step 8: missing/invalid auth on a protected route yields 401"]
async fn protected_routes_require_auth() {
    let app = test_app();
    let req = Request::get("/bookmarks").body(Body::empty()).unwrap();
    let resp = send(app.router.clone(), req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "step 8: POST /bookmarks validates the URL up front (400)"]
async fn create_rejects_bad_url() {
    let app = test_app();
    let req = Request::post("/bookmarks")
        .header("authorization", bearer())
        .header("content-type", "application/json")
        .body(Body::from(r#"{"url":"not-a-url"}"#))
        .unwrap();
    let resp = send(app.router.clone(), req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "step 8: POST /bookmarks stores Pending, returns 201, enqueues id"]
async fn create_stores_pending_and_enqueues() {
    let mut app = test_app();
    let req = Request::post("/bookmarks")
        .header("authorization", bearer())
        .header("content-type", "application/json")
        .body(Body::from(r#"{"url":"https://example.com","tags":["rust"]}"#))
        .unwrap();
    let resp = send(app.router.clone(), req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = body_string(resp).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "pending");
    assert_eq!(json["url"], "https://example.com");

    // The id was enqueued for the enrichment worker.
    let enqueued = app.enqueued.try_recv().expect("an id was enqueued");
    assert_eq!(enqueued, json["id"].as_str().unwrap());
}

#[tokio::test]
#[ignore = "step 8: GET /bookmarks lists, ?read=&tag= filters compose"]
async fn list_applies_filters() {
    let app = test_app();
    app.store.insert(mk("a", "https://a.test", &["rust"])).await.unwrap();
    app.store.insert(mk("b", "https://b.test", &["java"])).await.unwrap();

    let req = Request::get("/bookmarks?tag=rust")
        .header("authorization", bearer())
        .body(Body::empty())
        .unwrap();
    let resp = send(app.router.clone(), req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body_string(resp).await).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["id"], "a");
}

#[tokio::test]
#[ignore = "step 8: PATCH sets read, DELETE removes, unknown id -> 404"]
async fn patch_and_delete() {
    let app = test_app();
    app.store.insert(mk("a", "https://a.test", &[])).await.unwrap();

    let patch = Request::patch("/bookmarks/a")
        .header("authorization", bearer())
        .header("content-type", "application/json")
        .body(Body::from(r#"{"read":true}"#))
        .unwrap();
    assert_eq!(send(app.router.clone(), patch).await.status(), StatusCode::OK);
    assert!(app.store.get("a").await.unwrap().unwrap().read);

    let del = Request::delete("/bookmarks/a")
        .header("authorization", bearer())
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(app.router.clone(), del).await.status(), StatusCode::NO_CONTENT);

    let missing = Request::delete("/bookmarks/a")
        .header("authorization", bearer())
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(app.router.clone(), missing).await.status(), StatusCode::NOT_FOUND);
}

fn mk(id: &str, url: &str, tags: &[&str]) -> readlater::model::Bookmark {
    let _ = (TOKEN, BookmarkStatus::Pending);
    readlater::model::Bookmark::new(
        id.to_string(),
        url.to_string(),
        tags.iter().map(|s| s.to_string()).collect(),
        1,
    )
}
