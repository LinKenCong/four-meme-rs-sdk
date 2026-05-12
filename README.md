# four_meme_sdk

Rust SDK for integrating with Four.meme REST APIs and BSC contracts.

The crate exposes programmatic APIs that replace the TypeScript scripts from the Four.meme integration skill:

- Public REST: config, token detail, token search, token rankings.
- Token creation: nonce/login, image upload, create payload/signature preparation, and on-chain `createToken` submission.
- Trading: token info, buy/sell quotes, buy/sell execution with required approvals.
- Transfers: native BNB and ERC-20 transfers.
- Events: recent or ranged TokenManager2 event logs.
- EIP-8004: agent NFT balance and registration.
- Tax tokens: tax configuration queries.

## Quick Start

```rust
use four_meme_sdk::{FourMemeSdk, SdkConfig};

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = FourMemeSdk::new(SdkConfig::default())?;
    let config = sdk.public_config().await?;
    println!("raised tokens: {}", config.len());
    Ok(())
}
```

Transactions require a private key passed by the caller. The SDK never reads or stores `.env` values.
