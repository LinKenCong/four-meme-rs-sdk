# Release Checklist

Use this checklist before publishing `four_meme_sdk` or deploying it into a production bot/indexer.

## Required Local Gate

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-targets --all-features`
- [ ] `cargo check --examples --all-features`
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`
- [ ] Secret hygiene scan: `rg -n "(PRIVATE_KEY|meme-web-access|0x[a-fA-F0-9]{64})" .`

Expected secret-scan matches are limited to documentation warnings, protocol/header names, test hashes, and redaction fixtures.

## Public API Review

- [ ] REST reads return typed models (`PublicConfig`, `TokenDetail`, `TokenSearchResponse`, `TokenRankingResponse`) with compatibility fields for unknown API keys.
- [ ] Write methods return `ConfirmedReceipt` or result structs containing confirmed receipts.
- [ ] Trade planning APIs expose approval and execution boundaries before any transaction is submitted.
- [ ] Amount parsing paths use integer/decimal helpers instead of `f64` for user-supplied chain values.
- [ ] New public request/response and planning types are re-exported from the crate root.

## Mainnet Safety Review

- [ ] No examples broadcast transactions by default.
- [ ] Transaction examples require explicit signer environment variables and are documented for local forks or operator-confirmed flows.
- [ ] No real private keys, seed phrases, access tokens, or private RPC URLs are present.
- [ ] Receipt status failures map to `TransactionFailed` rather than successful hashes.
- [ ] Business API errors preserve redacted context without panicking when `data` is missing.

## Publishing Metadata

- [ ] `Cargo.toml` has `license`, `description`, `homepage`, `repository`, `documentation`, `readme`, `keywords`, `categories`, and `rust-version`.
- [ ] `README.md`, `SECURITY.md`, `CONTRIBUTING.md`, issue templates, PR template, and CI workflow are present.
- [ ] `cargo package --no-verify --list` contents are reviewed before any real publish command.
