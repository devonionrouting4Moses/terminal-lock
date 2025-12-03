use rand::prelude::IndexedRandom;
use std::time::Duration;
use terminal_lock_core::types::{Quote, QuoteSource};

use super::apis::{filter_apis_by_source, parse_response};

/// Quote fetcher with timeout
pub struct QuoteFetcher {
    timeout: Duration,
}

impl QuoteFetcher {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(3),
        }
    }

    /// Fetch quote from API
    pub async fn fetch(&self, source: QuoteSource) -> Quote {
        let apis = filter_apis_by_source(source);
        
        if apis.is_empty() {
            return crate::get_fallback_quote();
        }

        // Pick random API
        let api = apis.choose(&mut rand::rng()).unwrap();

        // Try to fetch with timeout
        match self.fetch_from_api(api.name, api.url).await {
            Ok(quote) => quote,
            Err(_) => crate::get_fallback_quote(),
        }
    }

    async fn fetch_from_api(&self, name: &str, url: &str) -> anyhow::Result<Quote> {
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()?;

        let response = client.get(url).send().await?;
        let body = response.text().await?;

        parse_response(name, &body).ok_or_else(|| anyhow::anyhow!("Failed to parse response"))
    }

    /// Fetch quote synchronously (blocking)
    pub fn fetch_blocking(&self, source: QuoteSource) -> Quote {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(self.fetch(source))
    }
}

impl Default for QuoteFetcher {
    fn default() -> Self {
        Self::new()
    }
}