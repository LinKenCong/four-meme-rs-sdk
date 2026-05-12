//! Programmatic Rust SDK for Four.meme REST APIs and BSC contracts.
//!
//! This crate replaces the original TypeScript utility scripts with reusable
//! Rust APIs for public REST reads, token creation preparation/submission,
//! trading quotes and execution, transfers, TokenManager2 events, tax tokens,
//! and EIP-8004 agent registration.
//!
//! # TypeScript script migration
//!
//! | TypeScript script | Rust SDK replacement | Coverage |
//! | --- | --- | --- |
//! | `get-public-config` | [`FourMemeSdk::public_config`] | Typed read API. |
//! | `token-list` | [`FourMemeSdk::token_search`] with [`TokenSearchRequest`] | Request typed; response is dynamic JSON. |
//! | `token-get` | [`FourMemeSdk::token_detail`] | Dynamic JSON response. |
//! | `token-rankings` | [`FourMemeSdk::token_rankings`] with [`RankingRequest`] | Request typed; response is dynamic JSON. |
//! | `quote-buy` | [`FourMemeSdk::quote_buy`] | Typed on-chain quote. |
//! | `quote-sell` | [`FourMemeSdk::quote_sell`] | Typed on-chain quote. |
//! | `execute-buy` | [`FourMemeSdk::execute_buy`] with [`BuyMode`] | Includes version check, approvals, and transaction submission. |
//! | `execute-sell` | [`FourMemeSdk::execute_sell`] | Includes approval and transaction submission. |
//! | `send-token` | [`FourMemeSdk::send_asset`] with [`Asset`] | Supports native BNB and ERC-20 transfers. |
//! | `create-token-api` | [`FourMemeSdk::prepare_create_token`] with [`CreateTokenRequest`] | Login, upload, create payload, signature, and fee preparation. |
//! | `create-token-chain` | [`FourMemeSdk::submit_create_token`] | Submits a reviewed create payload. |
//! | `create-token-instant` | [`FourMemeSdk::prepare_create_token`] plus [`FourMemeSdk::submit_prepared_create_token`] | Split so callers can review fees before broadcasting. |
//! | `get-token-info` | [`FourMemeSdk::get_token_info`] | Typed TokenManagerHelper3 state. |
//! | `get-tax-token-info` | [`FourMemeSdk::get_tax_token_info`] | Typed tax token state. |
//! | `get-recent-events` | [`FourMemeSdk::recent_events`] | Recent TokenManager2 logs. |
//! | `verify-events` | [`FourMemeSdk::events`] | Event retrieval only; assertions/reporting stay caller-owned. |
//! | `8004-balance` | [`FourMemeSdk::eip8004_balance`] | Typed NFT balance read. |
//! | `8004-register` | [`FourMemeSdk::register_agent`] | Transaction and metadata URI; decoded agent id is not yet modeled. |
//!
//! The SDK intentionally does not provide a CLI wrapper, load `.env` files, or
//! persist signing material. Pass secrets only to write methods from your own
//! secret-management boundary.
//!
//! # Read-only example
//!
//! ```no_run
//! use four_meme_sdk::{FourMemeSdk, SdkConfig, TokenSearchRequest};
//!
//! # async fn run() -> four_meme_sdk::Result<()> {
//! let sdk = FourMemeSdk::new(SdkConfig::default())?;
//! let request = TokenSearchRequest {
//!     keyword: Some("agent".to_string()),
//!     ..TokenSearchRequest::default()
//! };
//! let tokens = sdk.token_search(&request).await?;
//! println!("{tokens}");
//! # Ok(())
//! # }
//! ```
//!
//! # Write-flow example
//!
//! ```no_run
//! use four_meme_sdk::{CreateTokenApiOutput, FourMemeSdk, SdkConfig};
//!
//! # async fn run() -> four_meme_sdk::Result<()> {
//! let sdk = FourMemeSdk::new(SdkConfig::default())?;
//! let signing_secret = std::env::var("FOUR_MEME_SIGNER_SECRET").expect("set signing secret");
//! let prepared = CreateTokenApiOutput {
//!     create_arg: "<hex-or-base64-create-arg>".to_string(),
//!     signature: "<hex-or-base64-signature>".to_string(),
//!     creation_fee_wei: "0".to_string(),
//! };
//! let tx_hash = sdk.submit_prepared_create_token(signing_secret, &prepared).await?;
//! println!("submitted transaction: {tx_hash}");
//! # Ok(())
//! # }
//! ```

pub mod api;
pub mod client;
pub mod config;
pub mod contracts;
pub mod eip8004;
pub mod error;
pub mod events;
pub mod trade;
pub mod types;
pub mod utils;
pub mod wallet;

pub use client::FourMemeSdk;
pub use config::{Addresses, ConfigProfile, SdkConfig};
pub use error::{Result, SdkError};
pub use types::{
    AgentRegistration, Asset, BuyMode, BuyQuote, CreateTokenApiOutput, CreateTokenImage,
    CreateTokenRequest, CreateTokenResult, RaisedToken, RankingRequest, SellQuote, TaxTokenInfo,
    TokenEvent, TokenInfo, TokenLabel, TokenSearchRequest, TokenTaxInfo,
};
