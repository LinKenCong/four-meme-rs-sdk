use alloy::primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiEnvelope<T> {
    pub code: Value,
    pub msg: Option<String>,
    pub message: Option<String>,
    pub data: T,
}

impl<T> ApiEnvelope<T> {
    pub fn is_success(&self) -> bool {
        match &self.code {
            Value::String(code) => code == "0",
            Value::Number(code) => code.as_i64() == Some(0),
            _ => false,
        }
    }

    pub fn code_string(&self) -> String {
        match &self.code {
            Value::String(code) => code.clone(),
            other => other.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaisedToken {
    pub symbol: String,
    #[serde(default)]
    pub symbol_address: Option<String>,
    #[serde(default)]
    pub total_amount: Option<Value>,
    #[serde(default)]
    pub total_b_amount: Option<Value>,
    #[serde(default)]
    pub sale_rate: Option<Value>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenLabel {
    Meme,
    Ai,
    Defi,
    Games,
    Infra,
    DeSci,
    Social,
    Depin,
    Charity,
    Others,
}

impl TokenLabel {
    pub fn as_api_str(self) -> &'static str {
        match self {
            Self::Meme => "Meme",
            Self::Ai => "AI",
            Self::Defi => "Defi",
            Self::Games => "Games",
            Self::Infra => "Infra",
            Self::DeSci => "De-Sci",
            Self::Social => "Social",
            Self::Depin => "Depin",
            Self::Charity => "Charity",
            Self::Others => "Others",
        }
    }
}

impl TryFrom<&str> for TokenLabel {
    type Error = crate::error::SdkError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_ascii_lowercase().as_str() {
            "meme" => Ok(Self::Meme),
            "ai" => Ok(Self::Ai),
            "defi" => Ok(Self::Defi),
            "games" => Ok(Self::Games),
            "infra" => Ok(Self::Infra),
            "de-sci" | "desci" => Ok(Self::DeSci),
            "social" => Ok(Self::Social),
            "depin" => Ok(Self::Depin),
            "charity" => Ok(Self::Charity),
            "others" => Ok(Self::Others),
            other => Err(crate::error::SdkError::InvalidLabel(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTaxInfo {
    pub fee_rate: u16,
    pub burn_rate: u16,
    pub divide_rate: u16,
    pub liquidity_rate: u16,
    pub recipient_rate: u16,
    #[serde(default)]
    pub recipient_address: Option<String>,
    pub min_sharing: u64,
}

impl TokenTaxInfo {
    pub fn validate(&self) -> crate::Result<()> {
        let total = self.burn_rate + self.divide_rate + self.liquidity_rate + self.recipient_rate;
        if total != 100 {
            return Err(crate::SdkError::InvalidTaxRateSum(total));
        }
        if !matches!(self.fee_rate, 1 | 3 | 5 | 10) {
            return Err(crate::SdkError::InvalidTaxFeeRate(self.fee_rate));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenRequest {
    pub name: String,
    pub short_name: String,
    pub desc: String,
    pub label: String,
    pub image: CreateTokenImage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telegram_url: Option<String>,
    #[serde(default)]
    pub pre_sale: String,
    #[serde(default)]
    pub fee_plan: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_tax_info: Option<TokenTaxInfo>,
}

#[derive(Debug, Clone)]
pub enum CreateTokenImage {
    File { file_name: String, bytes: Vec<u8> },
    Url(String),
}

impl Serialize for CreateTokenImage {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            CreateTokenImage::File { file_name, .. } => serializer.serialize_str(file_name),
            CreateTokenImage::Url(url) => serializer.serialize_str(url),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenApiOutput {
    pub create_arg: String,
    pub signature: String,
    pub creation_fee_wei: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenResult {
    pub api: CreateTokenApiOutput,
    pub tx_hash: B256,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenSearchRequest {
    pub r#type: String,
    pub list_type: String,
    pub page_index: u32,
    pub page_size: u32,
    pub status: String,
    pub sort: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl Default for TokenSearchRequest {
    fn default() -> Self {
        Self {
            r#type: "HOT".to_string(),
            list_type: "NOR".to_string(),
            page_index: 1,
            page_size: 30,
            status: "PUBLISH".to_string(),
            sort: "DESC".to_string(),
            keyword: None,
            symbol: None,
            tag: None,
            version: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankingRequest {
    pub r#type: String,
    pub page_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_cap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_vol: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_vol: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_hold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_hold: Option<f64>,
}

impl RankingRequest {
    pub fn new(ranking_type: impl Into<String>) -> Self {
        Self {
            r#type: ranking_type.into(),
            page_size: 20,
            symbol: None,
            version: None,
            ranking_kind: None,
            min_cap: None,
            max_cap: None,
            min_vol: None,
            max_vol: None,
            min_hold: None,
            max_hold: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub version: u64,
    pub token_manager: Address,
    pub quote: Option<Address>,
    pub last_price: U256,
    pub trading_fee_rate: f64,
    pub min_trading_fee: U256,
    pub launch_time: u64,
    pub offers: U256,
    pub max_offers: U256,
    pub funds: U256,
    pub max_funds: U256,
    pub liquidity_added: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuyQuote {
    pub token_manager: Address,
    pub quote: Option<Address>,
    pub estimated_amount: U256,
    pub estimated_cost: U256,
    pub estimated_fee: U256,
    pub amount_msg_value: U256,
    pub amount_approval: U256,
    pub amount_funds: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SellQuote {
    pub token_manager: Address,
    pub quote: Option<Address>,
    pub funds: U256,
    pub fee: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxTokenInfo {
    pub fee_rate_bps: u64,
    pub fee_rate_percent: f64,
    pub rate_founder: u64,
    pub rate_holder: u64,
    pub rate_burn: u64,
    pub rate_liquidity: u64,
    pub min_dispatch: U256,
    pub min_share: U256,
    pub quote: Option<Address>,
    pub founder: Option<Address>,
}

#[derive(Debug, Clone, Copy)]
pub enum BuyMode {
    FixedAmount { amount: U256, max_funds: U256 },
    FixedFunds { funds: U256, min_amount: U256 },
}

#[derive(Debug, Clone, Copy)]
pub enum Asset {
    Native,
    Erc20(Address),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRegistration {
    pub tx_hash: B256,
    pub agent_id: Option<U256>,
    pub agent_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenEvent {
    pub event_name: String,
    pub block_number: u64,
    pub transaction_hash: B256,
    pub args: Value,
}
