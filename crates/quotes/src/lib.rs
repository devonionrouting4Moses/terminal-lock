pub mod apis;
pub mod fetcher;

pub use fetcher::QuoteFetcher;

use rand::prelude::IndexedRandom;
use terminal_lock_core::types::Quote;

/// Fallback quotes when API fails
pub fn get_fallback_quote() -> Quote {
    let fallbacks = vec![
        Quote::new(
            "The best way to predict the future is to invent it.".to_string(),
            Some("Alan Kay".to_string()),
        ),
        Quote::new(
            "Code is like humor. When you have to explain it, it's bad.".to_string(),
            Some("Cory House".to_string()),
        ),
        Quote::new(
            "First, solve the problem. Then, write the code.".to_string(),
            Some("John Johnson".to_string()),
        ),
        Quote::new(
            "Simplicity is the soul of efficiency.".to_string(),
            Some("Austin Freeman".to_string()),
        ),
        Quote::new(
            "Make it work, make it right, make it fast.".to_string(),
            Some("Kent Beck".to_string()),
        ),
        Quote::new(
            "Any fool can write code that a computer can understand. Good programmers write code that humans can understand.".to_string(),
            Some("Martin Fowler".to_string()),
        ),
    ];

    fallbacks
        .choose(&mut rand::rng())
        .cloned()
        .unwrap_or_else(Quote::fallback)
}