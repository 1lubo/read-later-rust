//! Step 5 — HTML metadata extraction (`extract_meta` in `src/fetcher.rs`).
//!
//! Pure + offline: we feed static HTML strings, no network. Unlock with
//! `cargo test --test step_05_fetcher -- --ignored`.
//!
//! Java/Spring: unit-testing a Jsoup-based parser — given a document, assert the
//! title/description it pulls out.

use readlater::fetcher::extract_meta;

#[test]
fn extracts_title() {
    let html = r#"<html><head><title>Hello World</title></head><body></body></html>"#;
    let meta = extract_meta(html);
    assert_eq!(meta.title.as_deref(), Some("Hello World"));
}

#[test]
fn excerpt_prefers_meta_description() {
    let html = r#"
        <html><head>
          <title>T</title>
          <meta name="description" content="the meta description">
          <meta property="og:description" content="the og description">
        </head><body><p>first paragraph</p></body></html>
    "#;
    let meta = extract_meta(html);
    assert_eq!(meta.excerpt.as_deref(), Some("the meta description"));
}

#[test]
fn excerpt_falls_back_to_og_description() {
    let html = r#"
        <html><head>
          <title>T</title>
          <meta property="og:description" content="the og description">
        </head><body><p>first paragraph</p></body></html>
    "#;
    let meta = extract_meta(html);
    assert_eq!(meta.excerpt.as_deref(), Some("the og description"));
}

#[test]
fn excerpt_falls_back_to_first_paragraph() {
    let html = r#"<html><head><title>T</title></head>
        <body><p>   </p><p>first real paragraph</p></body></html>"#;
    let meta = extract_meta(html);
    assert_eq!(meta.excerpt.as_deref(), Some("first real paragraph"));
}

#[test]
fn missing_fields_are_none() {
    let html = r#"<html><head></head><body></body></html>"#;
    let meta = extract_meta(html);
    assert_eq!(meta.title, None);
    assert_eq!(meta.excerpt, None);
}
