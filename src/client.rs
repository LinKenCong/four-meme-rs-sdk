use alloy::network::Ethereum;
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use reqwest::Client;
use url::Url;

use crate::config::{ConfigProfile, SdkConfig};
use crate::error::Result;

/// Four.meme SDK client for REST APIs and BSC contracts.
#[derive(Debug, Clone)]
pub struct FourMemeSdk {
    pub(crate) config: SdkConfig,
    pub(crate) http: Client,
    pub(crate) provider: DynProvider<Ethereum>,
}

impl FourMemeSdk {
    pub fn new(config: SdkConfig) -> Result<Self> {
        config.validate()?;
        let rpc_url = Url::parse(&config.rpc_url)?;
        let provider = ProviderBuilder::new().connect_http(rpc_url).erased();
        Ok(Self {
            config,
            http: Client::new(),
            provider,
        })
    }

    pub fn mainnet() -> Result<Self> {
        Self::new(SdkConfig::mainnet())
    }

    pub fn local_fork() -> Result<Self> {
        Self::new(SdkConfig::local_fork())
    }

    pub fn from_env() -> Result<Self> {
        Self::new(SdkConfig::from_env()?)
    }

    pub fn from_profile(profile: ConfigProfile) -> Result<Self> {
        Self::new(SdkConfig::from_profile(profile))
    }

    pub fn config(&self) -> &SdkConfig {
        &self.config
    }

    pub(crate) fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.config.api_base.trim_end_matches('/'), path)
    }

    pub(crate) fn signer_provider(
        &self,
        signer: PrivateKeySigner,
    ) -> Result<DynProvider<Ethereum>> {
        let rpc_url = Url::parse(&self.config.rpc_url)?;
        Ok(ProviderBuilder::new()
            .wallet(signer)
            .connect_http(rpc_url)
            .erased())
    }
}
