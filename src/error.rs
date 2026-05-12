use alloy::primitives::Address;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SdkError>;

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("invalid address `{0}`")]
    InvalidAddress(String),
    #[error("invalid contract address for `{field}`: zero address is not allowed")]
    InvalidContractAddress { field: &'static str },
    #[error("invalid config profile `{0}`; expected `mainnet` or `local-fork`")]
    InvalidConfigProfile(String),
    #[error("invalid api base url: {0}")]
    InvalidApiBaseUrl(String),
    #[error("invalid rpc url: {0}")]
    InvalidRpcUrl(String),
    #[error("invalid environment variable `{name}`: {reason}")]
    InvalidEnvVar { name: &'static str, reason: String },
    #[error("invalid private key")]
    InvalidPrivateKey,
    #[error("invalid amount `{0}`")]
    InvalidAmount(String),
    #[error("unsupported chain id {0}; Four.meme SDK currently supports BSC mainnet only")]
    UnsupportedChain(u64),
    #[error("unsupported token version {version}; expected {expected}")]
    UnsupportedTokenVersion { version: u64, expected: u64 },
    #[error("missing required field `{0}`")]
    MissingField(&'static str),
    #[error("invalid token label `{0}`")]
    InvalidLabel(String),
    #[error("tax rates must sum to 100, got {0}")]
    InvalidTaxRateSum(u16),
    #[error("tax fee rate must be one of 1, 3, 5, or 10; got {0}")]
    InvalidTaxFeeRate(u16),
    #[error("no raised token config is available")]
    MissingRaisedToken,
    #[error("invalid block range: from_block {from_block} is greater than to_block {to_block}")]
    InvalidBlockRange { from_block: u64, to_block: u64 },
    #[error("event block chunk size must be greater than 0, got {0}")]
    InvalidBlockChunkSize(u64),
    #[error("contract call failed: {0}")]
    Contract(String),
    #[error("transaction failed: {0}")]
    Transaction(String),
    #[error("api returned error code {code}: {body}")]
    Api { code: String, body: String },
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("url parse failed: {0}")]
    Url(#[from] url::ParseError),
    #[error("json failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("hex decode failed: {0}")]
    Hex(#[from] hex::FromHexError),
    #[error("wallet address {actual} does not match expected address {expected}")]
    WalletAddressMismatch { expected: Address, actual: Address },
}
