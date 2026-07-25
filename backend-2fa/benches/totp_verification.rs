use criterion::{criterion_group, criterion_main, Criterion};
use petchain_2fa::two_factor::TwoFactorAuth;

fn bench_generate_secret(c: &mut Criterion) {
    c.bench_function("generate_secret", |b| {
        b.iter(|| TwoFactorAuth::generate_secret())
    });
}

fn bench_setup(c: &mut Criterion) {
    c.bench_function("setup", |b| {
        b.iter(|| TwoFactorAuth::setup("bench@example.com", "PetChain").unwrap())
    });
}

fn bench_verify_token_invalid(c: &mut Criterion) {
    let setup = TwoFactorAuth::setup("bench@example.com", "PetChain").unwrap();
    let secret = setup.secret.clone();

    c.bench_function("verify_token_invalid", |b| {
        b.iter(|| TwoFactorAuth::verify_token(&secret, "000000"))
    });
}

fn bench_generate_backup_codes(c: &mut Criterion) {
    c.bench_function("generate_backup_codes_10", |b| {
        b.iter(|| TwoFactorAuth::generate_backup_codes(10))
    });
}

fn bench_verify_backup_code(c: &mut Criterion) {
    let codes = TwoFactorAuth::generate_backup_codes(10);
    let target = codes[5].clone();

    c.bench_function("verify_backup_code", |b| {
        b.iter(|| TwoFactorAuth::verify_backup_code(&codes, &target))
    });
}

criterion_group!(
    totp_benches,
    bench_generate_secret,
    bench_setup,
    bench_verify_token_invalid,
    bench_generate_backup_codes,
    bench_verify_backup_code,
);
criterion_main!(totp_benches);
