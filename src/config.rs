//! Configuration profiles, defaults, environment parsing, and contract address overrides.
//!
//! [`SdkConfig::mainnet`] targets Four.meme on BSC mainnet. [`SdkConfig::local_fork`] keeps BSC
//! chain id `56` and mainnet contract addresses while pointing RPC calls at a local fork.

use std::env;
use std::str::FromStr;
use std::time::Duration;

use alloy::primitives::{Address, address};
use url::Url;

use crate::error::{Result, SdkError};
use crate::utils::parse_address;

pub const BSC_CHAIN_ID: u64 = 56;
pub const DEFAULT_API_BASE: &str = "https://four.meme/meme-api/v1";
pub const DEFAULT_BSC_RPC_URL: &str = "https://bsc-dataseed.binance.org";
pub const LOCAL_FORK_RPC_URL: &str = "http://127.0.0.1:8545";
pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_IDEMPOTENT_RETRIES: u8 = 2;
pub const DEFAULT_USER_AGENT: &str = concat!("four_meme_sdk/", env!("CARGO_PKG_VERSION"));

const SDK_PROFILE_ENV: &str = "FOUR_MEME_PROFILE";
const SDK_API_BASE_ENV: &str = "FOUR_MEME_API_BASE";
const SDK_RPC_URL_ENV: &str = "FOUR_MEME_RPC_URL";
const SDK_CHAIN_ID_ENV: &str = "FOUR_MEME_CHAIN_ID";
const SDK_TOKEN_MANAGER2_ENV: &str = "FOUR_MEME_TOKEN_MANAGER2";
const SDK_TOKEN_MANAGER_HELPER3_ENV: &str = "FOUR_MEME_TOKEN_MANAGER_HELPER3";
const SDK_EIP8004_NFT_ENV: &str = "FOUR_MEME_EIP8004_NFT";

/// Built-in SDK configuration profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigProfile {
    /// BSC mainnet with public Four.meme API and contract addresses.
    Mainnet,
    /// Local BSC mainnet fork using the loopback RPC endpoint.
    LocalFork,
}

impl FromStr for ConfigProfile {
    type Err = SdkError;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "local-fork" | "local_fork" | "localfork" => Ok(Self::LocalFork),
            profile => Err(SdkError::InvalidConfigProfile(profile.to_string())),
        }
    }
}

/// Four.meme contract addresses used by the SDK.
#[derive(Debug, Clone)]
pub struct Addresses {
    pub token_manager2: Address,
    pub token_manager_helper3: Address,
    pub eip8004_nft: Address,
}

impl Default for Addresses {
    fn default() -> Self {
        Self::mainnet()
    }
}

impl Addresses {
    pub fn mainnet() -> Self {
        Self {
            token_manager2: address!("5c952063c7fc8610FFDB798152D69F0B9550762b"),
            token_manager_helper3: address!("F251F83e40a78868FcfA3FA4599Dad6494E46034"),
            eip8004_nft: address!("8004A169FB4a3325136EB29fA0ceB6D2e539a432"),
        }
    }

    pub fn local_fork() -> Self {
        Self::mainnet()
    }

    pub fn validate(&self) -> Result<()> {
        validate_contract_address("token_manager2", self.token_manager2)?;
        validate_contract_address("token_manager_helper3", self.token_manager_helper3)?;
        validate_contract_address("eip8004_nft", self.eip8004_nft)?;
        Ok(())
    }
}

/// Runtime configuration for REST and contract clients.
#[derive(Debug, Clone)]
pub struct SdkConfig {
    pub api_base: String,
    pub rpc_url: String,
    pub chain_id: u64,
    pub addresses: Addresses,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self::mainnet()
    }
}

impl SdkConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mainnet() -> Self {
        Self {
            api_base: DEFAULT_API_BASE.to_string(),
            rpc_url: DEFAULT_BSC_RPC_URL.to_string(),
            chain_id: BSC_CHAIN_ID,
            addresses: Addresses::mainnet(),
        }
    }

    pub fn local_fork() -> Self {
        Self {
            rpc_url: LOCAL_FORK_RPC_URL.to_string(),
            addresses: Addresses::local_fork(),
            ..Self::mainnet()
        }
    }

    pub fn from_profile(profile: ConfigProfile) -> Self {
        match profile {
            ConfigProfile::Mainnet => Self::mainnet(),
            ConfigProfile::LocalFork => Self::local_fork(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let profile = optional_env(SDK_PROFILE_ENV)?
            .map(|value| value.parse::<ConfigProfile>())
            .transpose()?
            .unwrap_or(ConfigProfile::Mainnet);
        let mut config = Self::from_profile(profile);

        if let Some(api_base) = optional_env(SDK_API_BASE_ENV)? {
            config.api_base = api_base;
        }
        if let Some(rpc_url) = optional_env(SDK_RPC_URL_ENV)? {
            config.rpc_url = rpc_url;
        }
        if let Some(chain_id) = optional_env(SDK_CHAIN_ID_ENV)? {
            config.chain_id = parse_chain_id(SDK_CHAIN_ID_ENV, &chain_id)?;
        }
        config.addresses = addresses_from_env(config.addresses)?;
        config.validate()?;
        Ok(config)
    }

    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = api_base.into();
        self
    }

    pub fn with_rpc_url(mut self, rpc_url: impl Into<String>) -> Self {
        self.rpc_url = rpc_url.into();
        self
    }

    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }

    pub fn with_addresses(mut self, addresses: Addresses) -> Self {
        self.addresses = addresses;
        self
    }

    pub fn validate(&self) -> Result<()> {
        if self.chain_id != BSC_CHAIN_ID {
            return Err(SdkError::UnsupportedChain(self.chain_id));
        }
        validate_api_base(&self.api_base)?;
        validate_rpc_url(&self.rpc_url)?;
        self.addresses.validate()?;
        Ok(())
    }
}

fn addresses_from_env(defaults: Addresses) -> Result<Addresses> {
    Ok(Addresses {
        token_manager2: optional_address(SDK_TOKEN_MANAGER2_ENV)?
            .unwrap_or(defaults.token_manager2),
        token_manager_helper3: optional_address(SDK_TOKEN_MANAGER_HELPER3_ENV)?
            .unwrap_or(defaults.token_manager_helper3),
        eip8004_nft: optional_address(SDK_EIP8004_NFT_ENV)?.unwrap_or(defaults.eip8004_nft),
    })
}

fn optional_address(name: &'static str) -> Result<Option<Address>> {
    optional_env(name)?.map(parse_address).transpose()
}

fn optional_env(name: &'static str) -> Result<Option<String>> {
    match env::var(name) {
        Ok(value) => Ok(normalize_optional_env(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(SdkError::InvalidEnvVar {
            name,
            reason: "value is not valid unicode".to_string(),
        }),
    }
}

fn normalize_optional_env(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn parse_chain_id(name: &'static str, value: &str) -> Result<u64> {
    value
        .parse::<u64>()
        .map_err(|error| SdkError::InvalidEnvVar {
            name,
            reason: format!("expected unsigned integer: {error}"),
        })
}

fn validate_api_base(api_base: &str) -> Result<()> {
    validate_http_url(api_base).map_err(SdkError::InvalidApiBaseUrl)
}

fn validate_rpc_url(rpc_url: &str) -> Result<()> {
    validate_http_url(rpc_url).map_err(SdkError::InvalidRpcUrl)
}

fn validate_http_url(value: &str) -> std::result::Result<(), String> {
    let url = Url::parse(value).map_err(|error| error.to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("scheme must be http or https".to_string());
    }
    if url.host_str().is_none() {
        return Err("host is required".to_string());
    }
    Ok(())
}

fn validate_contract_address(field: &'static str, address: Address) -> Result<()> {
    if address == Address::ZERO {
        return Err(SdkError::InvalidContractAddress { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_fork_uses_loopback_rpc_without_changing_chain() {
        let config = SdkConfig::local_fork();

        assert_eq!(config.rpc_url, LOCAL_FORK_RPC_URL);
        assert_eq!(config.chain_id, BSC_CHAIN_ID);
        assert_eq!(config.api_base, DEFAULT_API_BASE);
        assert_ne!(config.addresses.token_manager2, Address::ZERO);
    }

    #[test]
    fn profile_parser_accepts_documented_profiles() {
        assert_eq!(
            "mainnet".parse::<ConfigProfile>().unwrap(),
            ConfigProfile::Mainnet
        );
        assert_eq!(
            "local-fork".parse::<ConfigProfile>().unwrap(),
            ConfigProfile::LocalFork
        );
    }

    #[test]
    fn config_validation_rejects_zero_contract_addresses() {
        let config = SdkConfig::mainnet().with_addresses(Addresses {
            token_manager2: Address::ZERO,
            ..Addresses::mainnet()
        });

        assert!(matches!(
            config.validate(),
            Err(SdkError::InvalidContractAddress {
                field: "token_manager2"
            })
        ));
    }

    #[test]
    fn config_validation_rejects_non_http_rpc_url() {
        let config = SdkConfig::mainnet().with_rpc_url("wss://example.invalid");

        assert!(matches!(config.validate(), Err(SdkError::InvalidRpcUrl(_))));
    }

    #[test]
    fn config_validation_rejects_unsupported_chain_id() {
        let config = SdkConfig::mainnet().with_chain_id(97);

        assert!(matches!(
            config.validate(),
            Err(SdkError::UnsupportedChain(97))
        ));
    }
}
