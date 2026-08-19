//! Runtime configuration read from the environment (step 10).
//!
//! Java/Spring: this is your `application.yml` + `@ConfigurationProperties`.
//! Instead of a framework binding env vars for you, you read them yourself with
//! `std::env::var` and fail fast with a clear message when a required one is
//! missing.

/// All configuration the binary needs to boot.
pub struct Config {
    /// sqlx URL, e.g. `sqlite:/data/bookmarks.db?mode=rwc` or `sqlite::memory:`.
    pub database_url: String,
    /// Socket address to bind, e.g. `0.0.0.0:8080`.
    pub bind_addr: String,
    /// The single shared API/UI token clients must present.
    pub token: String,
    /// Secret bytes used to sign the session cookie (needs >= 64 bytes).
    pub cookie_key: Vec<u8>,
}

impl Config {
    /// Read config from the environment, applying sensible dev defaults for the
    /// non-secret values and requiring the secrets to be set explicitly.
    ///
    /// Java/Spring: the property binding + validation Spring Boot does at start.
    /// Breadcrumbs:
    /// - `std::env::var("DATABASE_URL")` (default `sqlite:bookmarks.db?mode=rwc`)
    /// - `std::env::var("BIND_ADDR")` (default `0.0.0.0:8080`)
    /// - `std::env::var("BOOKMARK_TOKEN")` (REQUIRED -> `Err` if missing)
    /// - `std::env::var("COOKIE_KEY")` (REQUIRED; `.into_bytes()`)
    pub fn from_env() -> Result<Self, String> {
        todo!("step 10: read DATABASE_URL/BIND_ADDR/BOOKMARK_TOKEN/COOKIE_KEY with defaults + required checks")
    }
}
