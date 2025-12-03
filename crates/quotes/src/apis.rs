use serde::Deserialize;
use terminal_lock_core::types::{Quote, QuoteSource};

/// Quote API endpoint
#[derive(Debug, Clone)]
pub struct QuoteApi {
    pub name: &'static str,
    pub url: &'static str,
    pub category: QuoteSource,
}

impl QuoteApi {
    pub const fn new(name: &'static str, url: &'static str, category: QuoteSource) -> Self {
        Self { name, url, category }
    }
}

/// All available quote APIs
pub fn get_quote_apis() -> Vec<QuoteApi> {
    vec![
        // Coding/Tech
        QuoteApi::new("Quotable Tech", "https://api.quotable.io/random?tags=technology", QuoteSource::Coding),
        QuoteApi::new("Quotable Science", "https://api.quotable.io/random?tags=science", QuoteSource::Coding),
        QuoteApi::new("Quotable Future", "https://api.quotable.io/random?tags=future", QuoteSource::Coding),
        
        // Spiritual/Wisdom
        QuoteApi::new("ZenQuotes", "https://zenquotes.io/api/random", QuoteSource::Spiritual),
        QuoteApi::new("Quotable Wisdom", "https://api.quotable.io/random?tags=wisdom", QuoteSource::Spiritual),
        QuoteApi::new("Quotable Philosophy", "https://api.quotable.io/random?tags=philosophy", QuoteSource::Spiritual),
        QuoteApi::new("Quotable Change", "https://api.quotable.io/random?tags=change", QuoteSource::Spiritual),
        QuoteApi::new("Quotable Courage", "https://api.quotable.io/random?tags=courage", QuoteSource::Spiritual),
        
        // Motivational
        QuoteApi::new("Quotable Motivational", "https://api.quotable.io/random?tags=motivational", QuoteSource::Motivational),
        QuoteApi::new("Quotable Inspirational", "https://api.quotable.io/random?tags=inspirational", QuoteSource::Motivational),
        QuoteApi::new("Quotable Success", "https://api.quotable.io/random?tags=success", QuoteSource::Motivational),
        
        // General
        QuoteApi::new("Quotable Random", "https://api.quotable.io/random", QuoteSource::General),
        QuoteApi::new("Advice Slip", "https://api.adviceslip.com/advice", QuoteSource::General),
        QuoteApi::new("Quotable Life", "https://api.quotable.io/random?tags=life", QuoteSource::General),
        QuoteApi::new("Quotable Famous", "https://api.quotable.io/random?tags=famous-quotes", QuoteSource::General),
    ]
}

/// Filter APIs by source
pub fn filter_apis_by_source(source: QuoteSource) -> Vec<QuoteApi> {
    let all_apis = get_quote_apis();
    
    match source {
        QuoteSource::All => all_apis,
        _ => all_apis.into_iter().filter(|api| api.category == source).collect(),
    }
}

// Response types for different APIs

#[derive(Debug, Deserialize)]
pub struct QuotableResponse {
    pub content: String,
    pub author: String,
}

#[derive(Debug, Deserialize)]
pub struct ZenQuotesResponse {
    pub q: String,
    pub a: String,
}

#[derive(Debug, Deserialize)]
pub struct AdviceSlipResponse {
    pub slip: AdviceSlip,
}

#[derive(Debug, Deserialize)]
pub struct AdviceSlip {
    pub advice: String,
}

/// Parse API response into Quote
pub fn parse_response(api_name: &str, body: &str) -> Option<Quote> {
    if api_name.contains("ZenQuotes") {
        // ZenQuotes returns array
        if let Ok(responses) = serde_json::from_str::<Vec<ZenQuotesResponse>>(body) {
            if let Some(resp) = responses.first() {
                return Some(Quote::new(resp.q.clone(), Some(resp.a.clone())));
            }
        }
    } else if api_name.contains("Advice Slip") {
        if let Ok(resp) = serde_json::from_str::<AdviceSlipResponse>(body) {
            return Some(Quote::new(resp.slip.advice, Some("Life Advice".to_string())));
        }
    } else {
        // Quotable API
        if let Ok(resp) = serde_json::from_str::<QuotableResponse>(body) {
            return Some(Quote::new(resp.content, Some(resp.author)));
        }
    }

    None
}