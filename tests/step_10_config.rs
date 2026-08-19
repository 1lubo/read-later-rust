//! Step 10 — configuration from the environment (`Config::from_env`).
//!
//! These mutate process env vars, so they run SERIALLY behind a mutex (env is
//! global, shared across threads in one test binary). Unlock with
//! `cargo test --test step_10_config -- --ignored`.
//!
//! Java/Spring: verifying property binding + fail-fast validation at startup.

use std::sync::Mutex;

use readlater::config::Config;

// Env is process-global; serialize the tests that touch it.
static ENV_LOCK: Mutex<()> = Mutex::new(());

const KEYS: [&str; 4] = ["DATABASE_URL", "BIND_ADDR", "BOOKMARK_TOKEN", "COOKIE_KEY"];

fn clear() {
    for k in KEYS {
        unsafe { std::env::remove_var(k) };
    }
}

#[test]
#[ignore = "step 10: required secrets missing -> Err"]
fn missing_required_secrets_is_err() {
    let _g = ENV_LOCK.lock().unwrap();
    clear();
    assert!(Config::from_env().is_err());
    clear();
}

#[test]
#[ignore = "step 10: secrets set -> Ok, with sensible defaults for the rest"]
fn defaults_apply_when_only_secrets_are_set() {
    let _g = ENV_LOCK.lock().unwrap();
    clear();
    unsafe {
        std::env::set_var("BOOKMARK_TOKEN", "tok");
        std::env::set_var("COOKIE_KEY", "0123456789012345678901234567890123456789012345678901234567890123");
    }

    let cfg = Config::from_env().expect("config ok");
    assert_eq!(cfg.token, "tok");
    assert!(!cfg.cookie_key.is_empty());
    // Defaults: a sqlite URL and a 0.0.0.0:8080 bind address.
    assert!(cfg.database_url.starts_with("sqlite:"));
    assert!(cfg.bind_addr.contains(":8080"));
    clear();
}

#[test]
#[ignore = "step 10: explicit DATABASE_URL / BIND_ADDR override the defaults"]
fn explicit_values_override_defaults() {
    let _g = ENV_LOCK.lock().unwrap();
    clear();
    unsafe {
        std::env::set_var("BOOKMARK_TOKEN", "tok");
        std::env::set_var("COOKIE_KEY", "0123456789012345678901234567890123456789012345678901234567890123");
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        std::env::set_var("BIND_ADDR", "127.0.0.1:9999");
    }

    let cfg = Config::from_env().expect("config ok");
    assert_eq!(cfg.database_url, "sqlite::memory:");
    assert_eq!(cfg.bind_addr, "127.0.0.1:9999");
    clear();
}
