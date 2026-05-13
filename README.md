# four_meme_sdk

Rust SDK for integrating with Four.meme REST APIs and BSC contracts.

The crate is designed for bots, indexers, dashboards, and local-fork test harnesses that need typed access to Four.meme data and explicit transaction workflows without porting one-off TypeScript scripts.

## Capabilities

- Public REST reads: platform config, token detail, token search, and rankings.
- Token creation: login nonce/signature, image upload or image URL payloads, create payload preparation, and `createToken` submission.
- Trading: token info, buy/sell quotes, quote-first execution plans, required ERC-20 approvals, and asset transfers.
- Events: recent or ranged TokenManager2 logs for creation, purchase, sale, and liquidity events.
- EIP-8004: agent NFT balance checks, metadata URI generation, registration, and registered agent id decoding.
- Tax tokens: tax configuration reads and token creation tax request validation.

## Safety Model

This SDK can submit irreversible BSC transactions when you call methods prefixed with `submit_`, `execute_`, `send_`, or `register_`. Read methods, quote methods, planning methods, and methods named `prepare_*` do not broadcast transactions.

- The SDK never reads `.env` files and never stores signer material.
- Pass signer keys explicitly from your own secret manager at the call boundary.
- Prefer `SdkConfig::local_fork()` or read-only quote flows while developing.
- Keep mainnet write flows behind explicit operator review of RPC URL, signer, target token, value, approval, and slippage.
- Do not commit real private keys, access tokens, seed phrases, private RPC URLs, or funded wallet details.

## Installation

Add the crate to your application:

```toml
[dependencies]
four_meme_sdk = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
alloy = { version = "1.0", features = ["primitives"] }
```

For local development from a checkout:

```toml
[dependencies]
four_meme_sdk = { path = "../four-meme-rs-sdk" }
```

## Configuration

`SdkConfig::mainnet()` is the default profile. It uses BSC chain id `56`, the public Four.meme REST API base URL, the default public BSC RPC URL, and known Four.meme contract addresses.

`SdkConfig::local_fork()` keeps chain id `56` and the same contract addresses but points RPC calls to `http://127.0.0.1:8545`. Use it with an Anvil, Hardhat, or Foundry fork when validating transaction flows without broadcasting to mainnet.

```rust
use four_meme_sdk::{FourMemeSdk, SdkConfig};

# fn main() -> four_meme_sdk::Result<()> {
let sdk = FourMemeSdk::new(
    SdkConfig::local_fork()
        .with_rpc_url("http://127.0.0.1:8545")
        .with_api_base("https://four.meme/meme-api/v1"),
)?;

assert_eq!(sdk.config().chain_id, four_meme_sdk::config::BSC_CHAIN_ID);
# Ok(())
# }
```

The SDK validates configuration before constructing clients. RPC and API URLs must use `http` or `https`; contract addresses must be non-zero; chain id must be `56`.

### Environment Loading

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

Transactions require a private key passed by the caller. Keep keys in your application secret manager or a local ignored `.env` file; the SDK never reads private-key variables or stores `.env` values.

## REST Reads

REST helpers decode Four.meme envelopes and return typed response models. Unknown response fields are preserved in `CompatibilityFields` so callers can inspect newly introduced API fields without waiting for a crate release.

```rust,no_run
use four_meme_sdk::{FourMemeSdk, RankingRequest, SdkConfig, TokenSearchRequest};

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = FourMemeSdk::new(SdkConfig::default())?;

    let config = sdk.public_config().await?;
    let detail = sdk
        .token_detail("0x0000000000000000000000000000000000000001".parse()?)
        .await?;
    let search = sdk.token_search(&TokenSearchRequest::default()).await?;
    let rankings = sdk.token_rankings(&RankingRequest::new("marketCap")).await?;

    println!("raised tokens: {}", config.len());
    println!("detail symbol: {:?}", detail.symbol);
    println!("search total: {:?}", search.total);
    println!("ranking entries: {}", rankings.list.len());
    Ok(())
}
```

Raw envelope helpers are available as `token_detail_raw`, `token_search_raw`, and `token_rankings_raw` when an integration needs the complete JSON body.

## Token Creation

Token creation is split into preparation and submission so applications can inspect cost, show confirmations, and dry-run before broadcasting.

```rust,no_run
use four_meme_sdk::{CreateTokenImage, CreateTokenRequest, FourMemeSdk, SdkConfig, TokenLabel};

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let signer_key = std::env::var("FOUR_MEME_SIGNER_KEY")
        .expect("load signer key from your secret manager");
    let sdk = FourMemeSdk::new(SdkConfig::default())?;

    let prepared = sdk
        .prepare_create_token(
            signer_key.as_str(),
            CreateTokenRequest {
                name: "Example Token".to_string(),
                short_name: "EXAMPLE".to_string(),
                desc: "Prepared by an SDK integration test wallet.".to_string(),
                label: TokenLabel::Meme,
                image: CreateTokenImage::Url("https://example.invalid/token.png".to_string()),
                web_url: None,
                twitter_url: None,
                telegram_url: None,
                pre_sale: "0".to_string(),
                fee_plan: false,
                token_tax_info: None,
            },
        )
        .await?;

    println!("creation fee wei: {}", prepared.creation_fee_wei);
    // Broadcast only on a local fork or after explicit operator confirmation:
    // let fork = FourMemeSdk::local_fork()?;
    // let receipt = fork.submit_prepared_create_token(signer_key, &prepared).await?;
    Ok(())
}
```

`prepare_create_token` validates required fields, signs the Four.meme login message, uploads file images when `CreateTokenImage::File` is used, prepares the API payload, normalizes `create_arg` and `signature`, and estimates the required creation fee. `submit_prepared_create_token` broadcasts the prepared arguments to TokenManager2 and returns a `ConfirmedReceipt` only after receipt status succeeds.

## Trading And Transfers

Use quote and planning methods before every write. Plans expose the token manager, quote response, required approval, `msg.value`, and exact contract call shape.

```rust,no_run
use alloy::primitives::U256;
use four_meme_sdk::{BuyMode, FourMemeSdk, SdkConfig};
use four_meme_sdk::utils::parse_address;

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = FourMemeSdk::new(SdkConfig::default())?;
    let token = parse_address("0x0000000000000000000000000000000000000001")?;
    let funds = U256::from(1_000_000_000_000_000_u64);
    let mode = BuyMode::FixedFunds {
        funds,
        min_amount: U256::from(1_u64),
    };

    let plan = sdk.plan_buy(token, mode).await?;
    println!("token manager: {}", plan.token_manager);
    println!("approval needed: {}", plan.approval.is_some());

    // On a local fork after reviewing `plan`:
    // let receipt = sdk.execute_buy("<signer-key-from-secret-manager>", token, mode).await?;
    Ok(())
}
```

Related helpers:

- `quote_buy` and `quote_sell` call TokenManagerHelper3 without broadcasting.
- `plan_buy` and `plan_sell` validate token version and build approval/execution plans.
- `approve_buy`, `approve_sell`, `execute_buy_plan`, and `execute_sell_plan` let applications split approvals from execution.
- `execute_buy` and `execute_sell` remain compatibility entry points that run approval plus execution and return `ConfirmedReceipt`.
- `send_asset` sends native BNB or an ERC-20 transfer and returns `ConfirmedReceipt`.
- `get_tax_token_info` reads tax-token fee and distribution settings.

## Events

Event helpers decode TokenManager2 log topics into typed events and preserve block/transaction metadata for indexing.

```rust,no_run
use four_meme_sdk::{FourMemeSdk, SdkConfig};

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = FourMemeSdk::new(SdkConfig::default())?;
    let events = sdk.recent_events(2_000).await?;

    for event in events {
        println!(
            "{} at block {:?}",
            event.event_name(),
            event.metadata.block_number
        );
    }
    Ok(())
}
```

Use `events(from_block, Some(to_block))` for deterministic backfills and `recent_events(block_count)` for monitoring. Large ranges are chunked internally to avoid oversized provider requests.

## EIP-8004 Agents

`build_agent_uri` creates the base64 `data:application/json` URI consumed by the EIP-8004 registration contract. Registration is a write operation, so prefer a fork first.

```rust,no_run
use four_meme_sdk::{AgentMetadata, Result};
use four_meme_sdk::eip8004::build_agent_uri;

fn main() -> Result<()> {
    let metadata = AgentMetadata::new(
        "Fork Bot",
        "https://example.invalid/agent.png",
        "Development-only Four.meme agent",
    )?;
    let uri = build_agent_uri(&metadata);
    assert!(uri.starts_with("data:application/json;base64,"));
    Ok(())
}
```

Use `eip8004_balance(owner)` for reads and `register_agent(signer_key, name, image_url, description)` for local-fork registration. Successful registration returns `AgentRegistration { tx_hash, agent_id, agent_uri }`.

## TypeScript Script Migration Matrix

| TypeScript script | Rust SDK replacement | Coverage | Notes |
| --- | --- | --- | --- |
| `get-public-config` | `FourMemeSdk::public_config()` | Covered | Returns `PublicConfig` with typed `RaisedToken` entries. |
| `token-list` | `FourMemeSdk::token_search(&TokenSearchRequest)` | Covered | Returns `TokenSearchResponse`; use `token_search_raw` for the full JSON envelope. |
| `token-get` | `FourMemeSdk::token_detail(address)` | Covered | Returns `TokenDetail`; use `token_detail_raw` for the full JSON envelope. |
| `token-rankings` | `FourMemeSdk::token_rankings(&RankingRequest)` | Covered | Returns `TokenRankingResponse`; supports `items`, `records`, `rows`, and list payloads. |
| `quote-buy` | `FourMemeSdk::quote_buy(token, amount, funds)` | Covered | Pass exactly one non-zero input; set the unused value to `U256::ZERO`. |
| `quote-sell` | `FourMemeSdk::quote_sell(token, amount)` | Covered | Uses TokenManagerHelper3 and validates non-zero amount. |
| `execute-buy` | `plan_buy` / `execute_buy` / `execute_buy_with_plan` | Covered | Includes version check, quote lookup, optional ERC-20 approval, and receipt status validation. |
| `execute-sell` | `plan_sell` / `execute_sell` / `execute_sell_with_plan` | Covered | Includes token approval, sell call selection, and receipt status validation. |
| `send-token` | `FourMemeSdk::send_asset(secret, to, amount, Asset)` | Covered | Supports native BNB and ERC-20 transfers. |
| `create-token-api` | `FourMemeSdk::prepare_create_token(secret, CreateTokenRequest)` | Covered | Login, optional image upload, create payload request, signature normalization, and fee estimation. |
| `create-token-chain` | `FourMemeSdk::submit_create_token(secret, create_arg, signature, value)` | Covered | Submits reviewed `createToken` calldata and caller-supplied value. |
| `create-token-instant` | `prepare_create_token` + `submit_prepared_create_token` | Covered | Keeps API preparation and on-chain submission explicit for fee review. |
| `get-token-info` | `FourMemeSdk::get_token_info(token)` | Covered | Typed TokenManagerHelper3 state. |
| `get-tax-token-info` | `FourMemeSdk::get_tax_token_info(token)` | Covered | Typed tax rates, thresholds, quote token, and founder address. |
| `get-recent-events` | `FourMemeSdk::recent_events(block_count)` | Covered | Fetches decoded TokenManager2 logs. |
| `verify-events` | `FourMemeSdk::events(from_block, to_block)` | Partial | Retrieval and decoding are covered; business assertions stay caller-owned. |
| `8004-balance` | `FourMemeSdk::eip8004_balance(owner)` | Covered | Reads EIP-8004 NFT balance. |
| `8004-register` | `FourMemeSdk::register_agent(secret, name, image_url, description)` | Covered | Validates metadata, broadcasts registration, checks receipt, and decodes `agent_id`. |

## Local Fork E2E Recipe

Docs in this repository avoid broadcasting to mainnet. For application-level end-to-end testing, run against a BSC fork and inject signer material from your secret manager at runtime.

1. Start a BSC fork that preserves chain id `56`.
2. Configure the SDK with `SdkConfig::local_fork().with_rpc_url("http://127.0.0.1:8545")`.
3. Run read paths first: `public_config`, `get_token_info`, `quote_buy`, and `recent_events`.
4. Build plans without broadcasting: `plan_buy`, `plan_sell`, and `prepare_create_token`.
5. Broadcast only fork transactions after asserting balances, allowances, slippage bounds, expected value, and expected revert handling.
6. Never commit real signer keys, API access tokens, fork snapshots containing secrets, or machine-specific absolute paths.

Repository quality gate:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo check --examples --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
rg -n "(PRIVATE_KEY|meme-web-access|0x[a-fA-F0-9]{64})" .
```

The final `rg` command should report only placeholders, protocol/header names, test fixtures, or documentation warnings.

## Examples

Compile-checked examples live in `examples/`:

- `public_config.rs`: read platform raised-token config.
- `token_search_detail.rs`: search tokens and fetch detail.
- `quotes.rs`: read token state and quote buy/sell paths.
- `events.rs`: fetch recent TokenManager2 events.
- `prepare_create.rs`: prepare token creation without broadcasting.
- `eip8004.rs`: read balance and register on a local fork when a key is explicitly provided.

Run examples with normal Cargo commands, for example:

```bash
cargo run --example public_config
cargo check --examples --all-features
```

## Module Map

- `api`: REST reads and token creation preparation.
- `client`: SDK construction, HTTP client options, retry policy, and provider setup.
- `config`: profiles, defaults, environment parsing, and address overrides.
- `contracts`: generated Alloy bindings for supported contracts.
- `trade`: token info, quotes, plans, writes, transfers, and tax-token reads.
- `events`: TokenManager2 log queries and decoding.
- `eip8004`: agent NFT reads, registration, and URI construction.
- `types`: request/response models shared across modules.
- `utils`: parsing and normalization helpers.
- `wallet`: signer creation and signer-address assertions.

## License

MIT
