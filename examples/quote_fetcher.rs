/// Example: Fetching quotes from various APIs
use terminal_lock_core::types::QuoteSource;
use terminal_lock_quotes::QuoteFetcher;

#[tokio::main]
async fn main() {
    let fetcher = QuoteFetcher::new();

    println!("Fetching quotes from different categories...\n");

    // Fetch from all categories
    let sources = vec![
        QuoteSource::All,
        QuoteSource::Coding,
        QuoteSource::Spiritual,
        QuoteSource::Motivational,
        QuoteSource::General,
    ];

    for source in sources {
        println!("=== {} ===", source);
        let quote = fetcher.fetch(source).await;
        println!("\"{}\"", quote.text);
        if let Some(author) = quote.author {
            println!("  — {}", author);
        }
        println!();
    }

    // Blocking fetch example
    println!("=== Blocking Fetch ===");
    let quote = fetcher.fetch_blocking(QuoteSource::Coding);
    println!("\"{}\"", quote.text);
    if let Some(author) = quote.author {
        println!("  — {}", author);
    }
}