use criterion::{black_box, criterion_group, criterion_main, Criterion};
use terminal_lock_core::types::QuoteSource;
use terminal_lock_quotes::QuoteFetcher;

fn benchmark_quote_fetching(c: &mut Criterion) {
    let fetcher = QuoteFetcher::new();
    
    c.bench_function("fetch_quote_blocking", |b| {
        b.iter(|| {
            fetcher.fetch_blocking(black_box(QuoteSource::All))
        })
    });
}

fn benchmark_fallback_quote(c: &mut Criterion) {
    c.bench_function("get_fallback_quote", |b| {
        b.iter(|| {
            terminal_lock_quotes::get_fallback_quote()
        })
    });
}

criterion_group!(
    benches,
    benchmark_quote_fetching,
    benchmark_fallback_quote
);
criterion_main!(benches);