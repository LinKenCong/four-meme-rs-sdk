# Contributing

Thank you for helping improve `four_meme_sdk`. This project aims to be safe by default for SDK consumers who integrate with Four.meme APIs and BSC contracts.

## Development Setup

1. Install a current stable Rust toolchain.
2. Clone the repository and install dependencies with Cargo.
3. Run the validation commands before opening a pull request:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo check --examples --all-features
```

If public Rust docs or doc comments changed, also run:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

## Security Expectations

Do not commit or paste real secrets anywhere in the repository.

- Use placeholders such as `<hex-encoded-test-key-placeholder>` and `<access-token-placeholder>` in examples.
- Keep `.env` files, wallet seed phrases, raw private keys, API tokens, keystore passwords, and production addresses out of commits.
- Avoid absolute local paths in docs, errors, tests, or generated fixtures.
- Redact secrets from issue logs, panic output, CI output, screenshots, and PR descriptions.
- Rotate any key or token that may have been exposed during development.

Before committing, run a local secret hygiene check:

```bash
rg -n "(PRIVATE_KEY|meme-web-access|0x[a-fA-F0-9]{64})" .
```

Expected matches must be placeholders, test fixtures, or documentation warnings only.

## Mainnet And Fork Testing

The SDK includes transaction-capable APIs. Pull requests must not require maintainers to spend mainnet funds to validate changes.

- Prefer read-only calls, unit tests, or forked-chain tests.
- Never run automated examples against a funded mainnet wallet.
- Document any transaction behavior with explicit warnings about BSC mainnet effects.
- For fork tests, use disposable accounts and deterministic setup data.
- Validate chain ID, RPC URL, signer address, target contract, token value, slippage, and approvals before broadcasting.

## Pull Request Checklist

Before requesting review, confirm that:

- [ ] The change has a clear feature/domain owner and does not mix unrelated concerns.
- [ ] Public APIs include appropriate docs and preserve compatibility unless the PR calls out a breaking change.
- [ ] Input validation, error handling, and edge cases are covered.
- [ ] Tests or documented validation steps cover the change.
- [ ] No real secrets, private keys, access tokens, seed phrases, or absolute user paths are included.
- [ ] Mainnet-writing behavior is either absent or clearly documented with safety warnings.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] `cargo test --all-targets --all-features` passes.
- [ ] `cargo check --examples --all-features` passes.

## Issue Guidance

Use the bug report or feature request templates when possible. For security vulnerabilities, follow `SECURITY.md` and report privately instead of opening a public issue.
