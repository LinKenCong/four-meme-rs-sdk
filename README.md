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
use four_meme_sdk::FourMemeSdk;

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = FourMemeSdk::mainnet()?;
    let config = sdk.public_config().await?;
    println!("raised tokens: {}", config.len());
    Ok(())
}
```

## Configuration Profiles

`SdkConfig::mainnet()` is the default production profile. It uses BSC mainnet chain id `56`, the public Four.meme API base URL, the public BSC RPC URL, and the known Four.meme contract addresses.

`SdkConfig::local_fork()` keeps the same chain id and contract addresses but points RPC calls to `http://127.0.0.1:8545`. Use this with an Anvil, Hardhat, or Foundry fork when validating transaction flows without broadcasting to mainnet:

```rust
use four_meme_sdk::FourMemeSdk;

let sdk = FourMemeSdk::local_fork()?;
assert_eq!(sdk.config().rpc_url, "http://127.0.0.1:8545");
```

For custom deployments or tests, build a config explicitly and let `FourMemeSdk::new` validate it before any provider is created:

```rust
use four_meme_sdk::{Addresses, FourMemeSdk, SdkConfig};

let config = SdkConfig::local_fork()
    .with_rpc_url("http://127.0.0.1:8545")
    .with_addresses(Addresses::mainnet());
let sdk = FourMemeSdk::new(config)?;
```

## Environment Loading

The SDK does not load `.env` files automatically. Applications that want environment-based configuration should load their `.env` file before calling `SdkConfig::from_env()` or `FourMemeSdk::from_env()`.

Supported optional variables:

| Variable | Purpose |
| --- | --- |
| `FOUR_MEME_PROFILE` | `mainnet` or `local-fork`; defaults to `mainnet`. |
| `FOUR_MEME_API_BASE` | Override the Four.meme REST API base URL. |
| `FOUR_MEME_RPC_URL` | Override the HTTP RPC endpoint. |
| `FOUR_MEME_CHAIN_ID` | Override the chain id; the SDK currently accepts only `56`. |
| `FOUR_MEME_TOKEN_MANAGER2` | Override the TokenManager2 contract address. |
| `FOUR_MEME_TOKEN_MANAGER_HELPER3` | Override the TokenManagerHelper3 contract address. |
| `FOUR_MEME_EIP8004_NFT` | Override the EIP-8004 NFT contract address. |

RPC and API URLs must use `http` or `https`, and contract addresses must be non-zero addresses.

Transactions require a private key passed by the caller. Keep keys in your application secret manager or local ignored `.env` file; the SDK never reads private-key variables or stores `.env` values.

Transactions require a signing secret passed by the caller. The SDK never reads or stores `.env` values.

## TypeScript Script Migration Matrix

The original TypeScript scripts lived in a local Four.meme integration skill. This SDK replaces them with reusable Rust APIs instead of one-off CLIs. Read-only APIs are safe to call by default. Write APIs only submit transactions when the caller explicitly passes a signing secret and invokes the submit method.

| TypeScript script | Rust SDK replacement | Coverage status | Notes |
| --- | --- | --- | --- |
| `get-public-config` | `FourMemeSdk::public_config()` | Covered | Returns typed `Vec<RaisedToken>` entries from `/public/config`. |
| `token-list` | `FourMemeSdk::token_search(&TokenSearchRequest)` | Covered with dynamic response | Request is typed; response remains `serde_json::Value` because the API payload has changed across releases. |
| `token-get` | `FourMemeSdk::token_detail(address)` | Covered with dynamic response | Returns the raw token detail JSON for callers that need fields not yet modeled. |
| `token-rankings` | `FourMemeSdk::token_rankings(&RankingRequest)` | Covered with dynamic response | Request is typed; ranking payload remains dynamic. |
| `quote-buy` | `FourMemeSdk::quote_buy(token, amount, funds)` | Covered | Uses `TokenManagerHelper3::tryBuy`; pass either token amount or funds as `U256::ZERO`. |
| `quote-sell` | `FourMemeSdk::quote_sell(token, amount)` | Covered | Uses `TokenManagerHelper3::trySell`. |
| `execute-buy` | `FourMemeSdk::execute_buy(secret, token, BuyMode)` | Covered | Handles version 2 validation, quote lookup, ERC-20 quote approval, and buy submission. |
| `execute-sell` | `FourMemeSdk::execute_sell(secret, token, amount, min_funds)` | Covered | Handles token approval and sell submission. |
| `send-token` | `FourMemeSdk::send_asset(secret, to, amount, Asset)` | Covered | Supports native BNB and ERC-20 transfers. |
| `create-token-api` | `FourMemeSdk::prepare_create_token(secret, CreateTokenRequest)` | Covered | Performs nonce/login, optional image upload, create payload request, signature normalization, and fee estimation. |
| `create-token-chain` | `FourMemeSdk::submit_create_token(secret, create_arg, signature, value)` | Covered | Submits the already prepared `createToken` calldata and caller-supplied value. |
| `create-token-instant` | `prepare_create_token` + `submit_prepared_create_token` | Covered | Keeps API preparation and on-chain submission explicit so production code can review the fee before broadcasting. |
| `get-token-info` | `FourMemeSdk::get_token_info(token)` | Covered | Returns typed launch/trading state from `TokenManagerHelper3`. |
| `get-tax-token-info` | `FourMemeSdk::get_tax_token_info(token)` | Covered | Returns typed tax rates, thresholds, quote token, and founder address. |
| `get-recent-events` | `FourMemeSdk::recent_events(block_count)` | Covered | Fetches recent `TokenCreate`, `TokenPurchase`, `TokenSale`, and `LiquidityAdded` logs. |
| `verify-events` | `FourMemeSdk::events(from_block, to_block)` | Partial | Event retrieval and naming are covered; assertion/report formatting remains caller-owned. |
| `8004-balance` | `FourMemeSdk::eip8004_balance(owner)` | Covered | Reads the EIP-8004 NFT balance. |
| `8004-register` | `FourMemeSdk::register_agent(secret, name, image_url, description)` | Partial | Registration transaction is covered; `agent_id` is currently `None` because log decoding is not yet modeled. |

### Coverage Gaps

- There is no bundled CLI wrapper. Applications should compose the SDK methods directly and own argument parsing, logging, retries, and process exit behavior.
- The SDK intentionally does not load `.env` files or manage secrets. Read signing material from your own secret manager and pass it only to write methods.
- Public token detail, search, and ranking responses are still returned as `serde_json::Value`; request builders and core chain-facing structures are typed.
- `verify-events` style business assertions are not hard-coded. Use `events` or `recent_events` and validate the returned `TokenEvent` list in your own tests.
- EIP-8004 registration currently returns the transaction hash and metadata URI, but not a decoded agent id.

## Read-Only Examples

Search published tokens:

```rust
use four_meme_sdk::{FourMemeSdk, SdkConfig, TokenSearchRequest};

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = FourMemeSdk::new(SdkConfig::default())?;
    let request = TokenSearchRequest {
        keyword: Some("agent".to_string()),
        ..TokenSearchRequest::default()
    };
    let tokens = sdk.token_search(&request).await?;
    println!("{tokens}");
    Ok(())
}
```

Quote a buy without broadcasting a transaction:

```rust
use alloy::primitives::{U256, address};
use four_meme_sdk::{FourMemeSdk, SdkConfig};

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = FourMemeSdk::new(SdkConfig::default())?;
    let token = address!("000000000000000000000000000000000000dEaD");
    let quote = sdk.quote_buy(token, U256::ZERO, U256::from(1_000_000_000_000_000u64)).await?;
    println!("estimated amount: {}", quote.estimated_amount);
    Ok(())
}
```

Fetch recent TokenManager2 events:

```rust
use four_meme_sdk::{FourMemeSdk, SdkConfig};

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = FourMemeSdk::new(SdkConfig::default())?;
    let events = sdk.recent_events(500).await?;
    for event in events {
        println!("{} in block {}", event.event_name, event.block_number);
    }
    Ok(())
}
```

## Write Flow Examples

The snippets below are intentionally split into preparation and submission steps. Use a non-production wallet, review all amounts, and avoid broadcasting on mainnet during tests.

Prepare a token creation payload without submitting the chain transaction:

```rust
use four_meme_sdk::{CreateTokenImage, CreateTokenRequest, FourMemeSdk, SdkConfig};

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = FourMemeSdk::new(SdkConfig::default())?;
    let signing_secret = std::env::var("FOUR_MEME_SIGNER_SECRET").expect("set signing secret");
    let request = CreateTokenRequest {
        name: "Example Token".to_string(),
        short_name: "EXAMPLE".to_string(),
        desc: "Example token created by an SDK integration test wallet".to_string(),
        label: "Meme".to_string(),
        image: CreateTokenImage::Url("https://example.invalid/token.png".to_string()),
        web_url: None,
        twitter_url: None,
        telegram_url: None,
        pre_sale: "0".to_string(),
        fee_plan: false,
        token_tax_info: None,
    };
    let prepared = sdk.prepare_create_token(signing_secret, request).await?;
    println!("creation fee wei: {}", prepared.creation_fee_wei);
    Ok(())
}
```

Submit a previously reviewed prepared payload:

```rust
use four_meme_sdk::{CreateTokenApiOutput, FourMemeSdk, SdkConfig};

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = FourMemeSdk::new(SdkConfig::default())?;
    let signing_secret = std::env::var("FOUR_MEME_SIGNER_SECRET").expect("set signing secret");
    let prepared = CreateTokenApiOutput {
        create_arg: "<hex-or-base64-create-arg>".to_string(),
        signature: "<hex-or-base64-signature>".to_string(),
        creation_fee_wei: "0".to_string(),
    };
    let tx_hash = sdk.submit_prepared_create_token(signing_secret, &prepared).await?;
    println!("submitted transaction: {tx_hash}");
    Ok(())
}
```
