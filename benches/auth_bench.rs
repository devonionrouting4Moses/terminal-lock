use criterion::{black_box, criterion_group, criterion_main, Criterion};
use terminal_lock_auth::{PasswordHasher, PasswordValidator};

fn benchmark_password_validation(c: &mut Criterion) {
    c.bench_function("validate_password", |b| {
        b.iter(|| {
            PasswordValidator::validate(black_box("Test123"))
        })
    });
}

fn benchmark_password_hashing(c: &mut Criterion) {
    let salt = PasswordHasher::generate_salt();
    
    c.bench_function("hash_password", |b| {
        b.iter(|| {
            PasswordHasher::hash_password(black_box("Test123"), &salt)
        })
    });
}

fn benchmark_password_verification(c: &mut Criterion) {
    let salt = PasswordHasher::generate_salt();
    let hash = PasswordHasher::hash_password("Test123", &salt).unwrap();
    
    c.bench_function("verify_password", |b| {
        b.iter(|| {
            PasswordHasher::verify_password(black_box("Test123"), &hash)
        })
    });
}

criterion_group!(
    benches,
    benchmark_password_validation,
    benchmark_password_hashing,
    benchmark_password_verification
);
criterion_main!(benches);