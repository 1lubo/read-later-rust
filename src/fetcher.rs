//! The `PageFetcher` seam: pull a page's title + excerpt.
//!
//! Step 5 lives here. `PageFetcher` is the dependency-inversion seam (like
//! Fragments' `Dispatcher`): tests use `FakeFetcher` (offline, canned) while
//! production uses `ReqwestFetcher` (real HTTP). The pure `extract_meta` helper
//! is unit-tested against static HTML — no network.

use std::collections::{HashMap, HashSet};

/// What we manage to pull out of a page.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageMeta {
    pub title: Option<String>,
    pub excerpt: Option<String>,
}

/// A failed fetch. These do NOT fail the request — the worker records them on
/// the bookmark as `Failed`.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("request failed: {0}")]
    Request(String),
    #[error("unexpected status: {0}")]
    Status(u16),
}

/// The seam. Java/Spring: an interface with one `PageMeta fetch(String url)`.
#[allow(async_fn_in_trait)]
pub trait PageFetcher {
    async fn fetch(&self, url: &str) -> Result<PageMeta, FetchError>;
}

/// Pure HTML → `PageMeta` extraction. Offline and unit-testable.
///
/// Java/Spring (Jsoup):
/// ```java
/// Document d = Jsoup.parse(html);
/// String title = d.title();
/// String desc = d.select("meta[name=description]").attr("content");
/// ```
/// Order for the excerpt: `<meta name="description">`, then
/// `<meta property="og:description">`, then the first non-empty `<p>`.
/// Use `scraper::{Html, Selector}`. Return `None` for anything missing.
pub fn extract_meta(html: &str) -> PageMeta {
    todo!("step 5: extract <title> + description/og:description/first <p> via scraper")
}

/// Real network fetcher. (Constructor provided; `fetch` is your step 5 work.)
pub struct ReqwestFetcher {
    client: reqwest::Client,
}

impl ReqwestFetcher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("readlater/0.1")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("failed to build reqwest client");
        Self { client }
    }
}

impl Default for ReqwestFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl PageFetcher for ReqwestFetcher {
    /// GET the URL, reject non-2xx as `FetchError::Status`, read the body text,
    /// then hand it to `extract_meta`.
    ///
    /// Java/Spring: `RestClient.get().uri(url).retrieve()...` then parse.
    async fn fetch(&self, url: &str) -> Result<PageMeta, FetchError> {
        let _ = &self.client; // step 5: use self.client.get(url).send().await ...
        todo!("step 5: fetch url, check status, read text, return extract_meta(&body)")
    }
}

/// Offline test double. Register canned metadata (or a forced failure) per URL.
/// Provided so worker/API tests stay deterministic and network-free.
#[derive(Clone, Default)]
pub struct FakeFetcher {
    responses: HashMap<String, PageMeta>,
    fail: HashSet<String>,
}

impl FakeFetcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a successful fetch for `url`.
    pub fn with(mut self, url: &str, title: Option<&str>, excerpt: Option<&str>) -> Self {
        self.responses.insert(
            url.to_string(),
            PageMeta {
                title: title.map(str::to_string),
                excerpt: excerpt.map(str::to_string),
            },
        );
        self
    }

    /// Register `url` to always fail.
    pub fn failing(mut self, url: &str) -> Self {
        self.fail.insert(url.to_string());
        self
    }
}

impl PageFetcher for FakeFetcher {
    async fn fetch(&self, url: &str) -> Result<PageMeta, FetchError> {
        if self.fail.contains(url) {
            return Err(FetchError::Request("forced failure".to_string()));
        }
        Ok(self.responses.get(url).cloned().unwrap_or_default())
    }
}
