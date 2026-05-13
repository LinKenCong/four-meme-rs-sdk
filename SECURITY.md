# Security Policy

## Supported Versions

This SDK is pre-1.0. Security fixes are provided for the latest commit on the default branch and the latest published crate version, when applicable.

## Reporting a Vulnerability

Please report suspected vulnerabilities privately. Do not open a public GitHub issue for security-sensitive reports.

1. Email the maintainers or use GitHub private vulnerability reporting if it is enabled for the repository.
2. Include the affected version or commit, a minimal reproduction, the expected impact, and any known mitigations.
3. Avoid including real private keys, access tokens, wallet seed phrases, or production account identifiers.
4. If a secret was exposed while reproducing the issue, rotate it before sending the report.

We aim to acknowledge reports within 3 business days and provide a remediation plan or status update within 7 business days.

## Private Key Handling

This crate signs BSC transactions only with keys supplied by the caller. Treat every private key as production-sensitive material.

- Load private keys from a secret manager, encrypted keystore, hardware wallet flow, or process environment controlled by your deployment platform.
- Never commit `.env` files, raw private keys, seed phrases, keystore passwords, or access tokens.
- Never paste real keys into GitHub issues, pull requests, logs, CI output, screenshots, or examples.
- Prefer ephemeral test wallets for development and rotate keys immediately after accidental exposure.
- Scope funded wallets to the minimum balance needed for the operation.
- Keep signing, transaction review, and broadcast boundaries explicit in application code.

Use placeholders in documentation and tests, for example:

```text
PRIVATE_KEY=<hex-encoded-test-key-placeholder>
MEME_WEB_ACCESS=<access-token-placeholder>
```

## Mainnet And Fork Safety

The SDK can create tokens, approve spenders, transfer assets, and execute trades. These operations can spend real BNB or tokens on BSC mainnet.

- Default local development to read-only API calls or a forked chain.
- Before sending a transaction, verify the RPC URL, chain ID, signer address, token address, spender address, slippage, deadline, gas settings, and value.
- Do not run automated examples against mainnet wallets with meaningful balances.
- When testing transaction flows, use a fork with deterministic fixtures or a disposable wallet with limited funds.
- Make dry-run, simulation, or explicit confirmation steps part of downstream applications.

## Dependency And Supply Chain Hygiene

- Review dependency updates before merging and prefer minimal feature sets.
- Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-targets --all-features` before release.
- Audit new code paths that parse external input, construct transactions, or handle signing material.
