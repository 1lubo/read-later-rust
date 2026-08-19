//! Authentication (step 7): a single shared token, presented either as an
//! `Authorization: Bearer <token>` header (JSON API) or a signed `session`
//! cookie (web UI).
//!
//! Java/Spring: `AuthToken` is a custom `HandlerMethodArgumentResolver` /
//! `OncePerRequestFilter` — a request either carries a valid credential or it
//! is rejected with 401 before your handler runs. `token_matches` uses a
//! CONSTANT-TIME compare (`subtle`) so timing can't leak the secret, the way
//! you'd reach for `MessageDigest.isEqual` rather than `String.equals`.

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum_extra::extract::cookie::Key;

use crate::error::AppError;
use crate::state::AppState;

/// The configured secret token plus the cookie-signing key. `Clone` because it
/// lives inside `AppState`. Fields are private: comparisons go through
/// `token_matches` so nobody accidentally does a non-constant-time `==`.
#[derive(Clone)]
pub struct AuthConfig {
    token: String,
    key: Key,
}

impl AuthConfig {
    /// `cookie_key` must be at least 64 bytes (`Key::from` panics otherwise).
    pub fn new(token: String, cookie_key: &[u8]) -> Self {
        Self {
            token,
            key: Key::from(cookie_key),
        }
    }

    /// The signing key, handed to `SignedCookieJar` via `FromRef` below.
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// Constant-time equality against the configured token.
    ///
    /// Breadcrumb: `use subtle::ConstantTimeEq;` then compare the two byte
    /// slices and fold the resulting `Choice` into a `bool` — but ONLY when the
    /// lengths match first, since the compare needs equal-length inputs.
    pub fn token_matches(&self, candidate: &str) -> bool {
        let _ = (&self.token, candidate);
        todo!("step 7: constant-time compare candidate against self.token (subtle::ConstantTimeEq)")
    }
}

/// Gives `SignedCookieJar` its signing key straight from `AppState`. (Provided.)
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.auth.key().clone()
    }
}

/// Presence of this extractor in a handler's arguments means "this route is
/// authenticated". Extraction fails with 401 if no valid credential is found.
///
/// Java/Spring: a controller method parameter resolved by a security filter —
/// if the filter rejects, the handler body never executes.
pub struct AuthToken;

impl FromRequestParts<AppState> for AuthToken {
    type Rejection = AppError;

    /// Accept the request if EITHER:
    /// - `Authorization: Bearer <token>` header matches, OR
    /// - the signed `session` cookie's value matches;
    /// otherwise return `Err(AppError::Unauthorized)`.
    ///
    /// Breadcrumbs: read `parts.headers` for `authorization`, or build a
    /// `SignedCookieJar::from_headers(&parts.headers, state.auth.key())` and
    /// read the `session` cookie; validate with `state.auth.token_matches(..)`.
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let _ = (parts, state);
        todo!("step 7: accept a matching Bearer header OR signed session cookie; else Unauthorized")
    }
}
