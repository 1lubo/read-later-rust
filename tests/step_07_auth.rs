//! Step 7 — token comparison (`AuthConfig::token_matches` in `src/auth.rs`).
//!
//! The full `AuthToken` extractor (Bearer header / signed cookie -> 401) is
//! exercised end-to-end in step 8's API tests; here we pin down the constant-
//! time token check on its own. Unlock with
//! `cargo test --test step_07_auth -- --ignored`.
//!
//! Java/Spring: unit-testing the credential comparison a security filter uses,
//! the way you'd test that it relies on `MessageDigest.isEqual` semantics.

use readlater::auth::AuthConfig;

// Cookie key must be >= 64 bytes for `Key::from`.
const COOKIE_KEY: &[u8] = &[7u8; 64];

#[test]
fn accepts_the_configured_token() {
    let auth = AuthConfig::new("s3cr3t-token".to_string(), COOKIE_KEY);
    assert!(auth.token_matches("s3cr3t-token"));
}

#[test]
fn rejects_a_wrong_token() {
    let auth = AuthConfig::new("s3cr3t-token".to_string(), COOKIE_KEY);
    assert!(!auth.token_matches("wrong"));
    assert!(!auth.token_matches("s3cr3t-toke")); // prefix, different length
    assert!(!auth.token_matches("s3cr3t-token-extra")); // superset
    assert!(!auth.token_matches("")); // empty
}
