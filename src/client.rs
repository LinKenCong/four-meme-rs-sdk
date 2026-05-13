use alloy::network::Ethereum;
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

use crate::config::{
    ConfigProfile, DEFAULT_CONNECT_TIMEOUT, DEFAULT_HTTP_TIMEOUT, DEFAULT_IDEMPOTENT_RETRIES,
    DEFAULT_USER_AGENT, SdkConfig,
};
use crate::error::{Result, SdkError};

/// Four.meme SDK client for REST APIs and BSC contracts.
#[derive(Debug, Clone)]
pub struct FourMemeSdk {
    pub(crate) config: SdkConfig,
    pub(crate) http: Client,
    pub(crate) provider: DynProvider<Ethereum>,
    retry_policy: RetryPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    max_retries: u8,
    base_delay: Duration,
}

impl RetryPolicy {
    pub fn disabled() -> Self {
        Self {
            max_retries: 0,
            base_delay: Duration::from_millis(0),
        }
    }

    pub fn idempotent(max_retries: u8) -> Self {
        Self {
            max_retries,
            base_delay: Duration::from_millis(100),
        }
    }

    fn delay_for_attempt(self, attempt: u8) -> Duration {
        self.base_delay
            .saturating_mul(2_u32.saturating_pow(attempt.into()))
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::idempotent(DEFAULT_IDEMPOTENT_RETRIES)
    }
}

#[derive(Debug, Clone)]
pub struct FourMemeSdkBuilder {
    config: SdkConfig,
    http: Option<Client>,
    timeout: Duration,
    connect_timeout: Duration,
    user_agent: String,
    retry_policy: RetryPolicy,
}

impl FourMemeSdkBuilder {
    pub fn new() -> Self {
        Self {
            config: SdkConfig::default(),
            http: None,
            timeout: DEFAULT_HTTP_TIMEOUT,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            user_agent: DEFAULT_USER_AGENT.to_string(),
            retry_policy: RetryPolicy::default(),
        }
    }

    pub fn config(mut self, config: SdkConfig) -> Self {
        self.config = config;
        self
    }
    pub fn api_base(mut self, api_base: impl Into<String>) -> Self {
        self.config.api_base = api_base.into();
        self
    }
    pub fn rpc_url(mut self, rpc_url: impl Into<String>) -> Self {
        self.config.rpc_url = rpc_url.into();
        self
    }
    pub fn chain_id(mut self, chain_id: u64) -> Self {
        self.config.chain_id = chain_id;
        self
    }
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = connect_timeout;
        self
    }
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }
    pub fn reqwest_client(mut self, http: Client) -> Self {
        self.http = Some(http);
        self
    }
    pub fn retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }
    pub fn idempotent_retries(mut self, max_retries: u8) -> Self {
        self.retry_policy = RetryPolicy::idempotent(max_retries);
        self
    }

    pub fn build(self) -> Result<FourMemeSdk> {
        self.config.validate()?;
        validate_http_options(self.timeout, self.connect_timeout, &self.user_agent)?;
        let rpc_url = Url::parse(&self.config.rpc_url)?;
        let provider = ProviderBuilder::new().connect_http(rpc_url).erased();
        let http = match self.http {
            Some(http) => http,
            None => Client::builder()
                .timeout(self.timeout)
                .connect_timeout(self.connect_timeout)
                .user_agent(self.user_agent)
                .build()?,
        };
        Ok(FourMemeSdk {
            config: self.config,
            http,
            provider,
            retry_policy: self.retry_policy,
        })
    }
}

impl Default for FourMemeSdkBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FourMemeSdk {
    pub fn new(config: SdkConfig) -> Result<Self> {
        Self::builder().config(config).build()
    }

    pub fn builder() -> FourMemeSdkBuilder {
        FourMemeSdkBuilder::new()
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

    pub(crate) async fn send_api_request(&self, request: RequestBuilder) -> Result<Response> {
        let method = request_method(&request);
        if !method.as_ref().is_some_and(is_idempotent_method) {
            return Ok(request.send().await?);
        }
        let Some(template) = request.try_clone() else {
            return Ok(request.send().await?);
        };
        let mut attempt = 0;
        loop {
            let Some(current) = template.try_clone() else {
                return Err(SdkError::InvalidHttpConfig(
                    "request body cannot be retried safely".to_string(),
                ));
            };
            match current.send().await {
                Ok(response)
                    if should_retry_status(response.status())
                        && attempt < self.retry_policy.max_retries =>
                {
                    sleep(self.retry_policy.delay_for_attempt(attempt)).await;
                    attempt += 1;
                }
                Ok(response) => return Ok(response),
                Err(error)
                    if is_retryable_error(&error) && attempt < self.retry_policy.max_retries =>
                {
                    sleep(self.retry_policy.delay_for_attempt(attempt)).await;
                    attempt += 1;
                }
                Err(error) => return Err(error.into()),
            }
        }
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

fn request_method(request: &RequestBuilder) -> Option<Method> {
    request
        .try_clone()?
        .build()
        .ok()
        .map(|request| request.method().clone())
}

fn validate_http_options(
    timeout: Duration,
    connect_timeout: Duration,
    user_agent: &str,
) -> Result<()> {
    if timeout.is_zero() {
        return Err(SdkError::InvalidHttpConfig(
            "timeout must be greater than zero".to_string(),
        ));
    }
    if connect_timeout.is_zero() {
        return Err(SdkError::InvalidHttpConfig(
            "connect timeout must be greater than zero".to_string(),
        ));
    }
    if user_agent.trim().is_empty() {
        return Err(SdkError::InvalidHttpConfig(
            "user agent must not be empty".to_string(),
        ));
    }
    Ok(())
}

fn is_idempotent_method(method: &Method) -> bool {
    matches!(method.as_str(), "GET" | "HEAD" | "OPTIONS" | "TRACE")
}

fn is_retryable_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect()
}

fn should_retry_status(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}
