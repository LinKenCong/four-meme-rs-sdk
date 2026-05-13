//! Production-oriented Rust SDK for Four.meme REST APIs and BSC contracts.
//!
//! Start with [`FourMemeSdk`] and [`SdkConfig`], then use the module that matches
//! your workflow:
//!
//! - [`api`]: typed REST reads plus token creation preparation.
//! - [`trade`]: token info, buy/sell quotes, execution plans, submissions, transfers, and tax-token reads.
//! - [`events`]: TokenManager2 log queries and typed event decoding for indexers.
//! - [`eip8004`]: agent NFT balance checks, registration, and metadata URI construction.
//! - [`utils`]: address, decimal amount, and payload parsing helpers.
//! - [`wallet`]: local signer construction and signer-address checks.
//!
//! # Safety model
//!
//! Read methods, quote methods, planning methods, and methods named `prepare_*` do not broadcast
//! transactions. Methods named `submit_*`, `execute_*`, `send_*`, or `register_*` can submit
//! irreversible BSC transactions. The SDK never reads `.env` files or stores private keys; callers
//! must inject signer material from their own secret-management boundary.
//!
//! Prefer local forks for development and keep mainnet write flows behind explicit operator
//! confirmation. A local fork must preserve BSC chain id `56`; other chain ids are rejected with
//! [`SdkError::UnsupportedChain`].
//!
//! # REST reads
//!
//! ```rust,no_run
//! use four_meme_sdk::{FourMemeSdk, RankingRequest, SdkConfig, TokenSearchRequest};
//!
//! # async fn run() -> four_meme_sdk::Result<()> {
//! let sdk = FourMemeSdk::new(SdkConfig::default())?;
//! let config = sdk.public_config().await?;
//! let search = sdk.token_search(&TokenSearchRequest::default()).await?;
//! let rankings = sdk.token_rankings(&RankingRequest::new("marketCap")).await?;
//!
//! println!("raised tokens: {}", config.len());
//! println!("search total: {:?}", search.total);
//! println!("ranking entries: {}", rankings.list.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Quote-first trading
//!
//! ```rust,no_run
//! use alloy::primitives::U256;
//! use four_meme_sdk::{BuyMode, FourMemeSdk, SdkConfig};
//! use four_meme_sdk::utils::parse_address;
//!
//! # async fn run() -> four_meme_sdk::Result<()> {
//! let sdk = FourMemeSdk::new(SdkConfig::default())?;
//! let token = parse_address("0x0000000000000000000000000000000000000001")?;
//! let mode = BuyMode::FixedFunds {
//!     funds: U256::from(1_000_000_000_000_000_u64),
//!     min_amount: U256::from(1_u64),
//! };
//! let plan = sdk.plan_buy(token, mode).await?;
//! println!("approval required: {}", plan.approval.is_some());
//! # Ok(())
//! # }
//! ```
//!
//! See the repository README for installation, local-fork E2E guidance, examples, and the
//! TypeScript script migration matrix.

pub use alloy::signers::local::PrivateKeySigner;

pub mod api;
pub mod client;
pub mod config;
pub mod contracts;
pub mod eip8004;
pub mod error;
pub mod events;
pub mod receipt;
pub mod trade;
pub mod types;
pub mod utils;
pub mod wallet;

pub use client::{FourMemeSdk, FourMemeSdkBuilder, RetryPolicy};
pub use config::{Addresses, ConfigProfile, SdkConfig};
pub use error::{Result, SdkError};
pub use types::{
    AgentMetadata, AgentRegistration, ApiCode, Asset, BuyExecutionPlan, BuyExecutionResult,
    BuyMode, BuyPlan, BuyQuote, CompatibilityFields, ConfirmedReceipt, CreateTokenApiOutput,
    CreateTokenImage, CreateTokenRequest, CreateTokenResult, PublicConfig, RaisedToken,
    RankingRequest, SellExecutionPlan, SellExecutionResult, SellPlan, SellQuote, TaxTokenInfo,
    TokenDetail, TokenEvent, TokenInfo, TokenLabel, TokenRankingEntry, TokenRankingResponse,
    TokenSearchRequest, TokenSearchResponse, TokenSummary, TokenTaxInfo, TradeApproval,
    TradeApprovalReceipt, TradeExecutionReceipt,
};
pub use wallet::{assert_signer_address, signer_from_private_key};
