## Summary

- 

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-targets --all-features`
- [ ] `cargo check --examples --all-features`
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` (if public docs changed)
- [ ] Secret hygiene check: `rg -n "(PRIVATE_KEY|meme-web-access|0x[a-fA-F0-9]{64})" .`

## Security And Mainnet Safety

- [ ] No real private keys, access tokens, seed phrases, keystore passwords, or absolute user paths are included.
- [ ] Any transaction-capable behavior clearly warns about BSC mainnet effects.
- [ ] Fork or transaction tests use disposable accounts and do not require funded mainnet wallets.
- [ ] Logs, screenshots, fixtures, and error messages redact sensitive data.

## Notes For Reviewers

- Breaking changes:
- Follow-up work:
