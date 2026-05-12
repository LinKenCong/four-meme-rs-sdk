use alloy::primitives::{Address, address};

pub const BSC_CHAIN_ID: u64 = 56;
pub const DEFAULT_API_BASE: &str = "https://four.meme/meme-api/v1";
pub const DEFAULT_BSC_RPC_URL: &str = "https://bsc-dataseed.binance.org";

#[derive(Debug, Clone)]
pub struct Addresses {
    pub token_manager2: Address,
    pub token_manager_helper3: Address,
    pub eip8004_nft: Address,
}

impl Default for Addresses {
    fn default() -> Self {
        Self {
            token_manager2: address!("5c952063c7fc8610FFDB798152D69F0B9550762b"),
            token_manager_helper3: address!("F251F83e40a78868FcfA3FA4599Dad6494E46034"),
            eip8004_nft: address!("8004A169FB4a3325136EB29fA0ceB6D2e539a432"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SdkConfig {
    pub api_base: String,
    pub rpc_url: String,
    pub chain_id: u64,
    pub addresses: Addresses,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            api_base: DEFAULT_API_BASE.to_string(),
            rpc_url: DEFAULT_BSC_RPC_URL.to_string(),
            chain_id: BSC_CHAIN_ID,
            addresses: Addresses::default(),
        }
    }
}

impl SdkConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = api_base.into();
        self
    }

    pub fn with_rpc_url(mut self, rpc_url: impl Into<String>) -> Self {
        self.rpc_url = rpc_url.into();
        self
    }

    pub fn with_addresses(mut self, addresses: Addresses) -> Self {
        self.addresses = addresses;
        self
    }
}
