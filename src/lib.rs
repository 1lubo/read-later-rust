//! Library crate root.
//!
//! Java/Spring parallel: this is where we declare which "packages" (modules)
//! exist. Declaring them here makes them visible to the binary (`main.rs`) AND
//! to the integration tests in `tests/` — a test can only see items exposed
//! through the library crate.

pub mod api;
pub mod auth;
pub mod config;
pub mod error;
pub mod fetcher;
pub mod in_memory;
pub mod model;
pub mod sqlite;
pub mod state;
pub mod store;
pub mod web;
pub mod worker;
