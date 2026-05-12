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

    pub(crate) fn message_text(&self) -> String {
        self.msg
            .as_deref()
            .or(self.message.as_deref())
            .unwrap_or("request failed")
            .to_string()
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
            other => Err(crate::error::SdkError::validation(
                "label",
                format!("invalid token label `{other}`"),
            )),
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
            return Err(crate::SdkError::validation(
                "token_tax_info",
                format!("tax rates must sum to 100, got {total}"),
            ));
        }
        if !matches!(self.fee_rate, 1 | 3 | 5 | 10) {
            return Err(crate::SdkError::validation(
                "fee_rate",
                format!(
                    "tax fee rate must be one of 1, 3, 5, or 10; got {}",
                    self.fee_rate
                ),
            ));
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
    pub metadata: TokenEventMetadata,
    pub kind: TokenManagerEvent,
}

impl TokenEvent {
    pub fn event_name(&self) -> &'static str {
        self.kind.event_name()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenEventMetadata {
    pub address: Address,
    pub block_hash: Option<B256>,
    pub block_number: Option<u64>,
    pub block_timestamp: Option<u64>,
    pub transaction_hash: Option<B256>,
    pub transaction_index: Option<u64>,
    pub log_index: Option<u64>,
    pub is_removed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "eventName", content = "args")]
pub enum TokenManagerEvent {
    TokenCreate(TokenCreateEvent),
    TokenPurchase(TokenPurchaseEvent),
    TokenSale(TokenSaleEvent),
    LiquidityAdded(LiquidityAddedEvent),
    Raw(RawTokenManagerEvent),
}

impl TokenManagerEvent {
    pub fn event_name(&self) -> &'static str {
        match self {
            Self::TokenCreate(_) => "TokenCreate",
            Self::TokenPurchase(_) => "TokenPurchase",
            Self::TokenSale(_) => "TokenSale",
            Self::LiquidityAdded(_) => "LiquidityAdded",
            Self::Raw(_) => "Raw",
        }
    }

    pub fn signature_hashes() -> Vec<B256> {
        use alloy::sol_types::SolEvent;

        vec![
            crate::contracts::TokenManager2::TokenCreate::SIGNATURE_HASH,
            crate::contracts::TokenManager2::TokenPurchase::SIGNATURE_HASH,
            crate::contracts::TokenManager2::TokenSale::SIGNATURE_HASH,
            crate::contracts::TokenManager2::LiquidityAdded::SIGNATURE_HASH,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenCreateEvent {
    pub creator: Address,
    pub token: Address,
    pub request_id: U256,
    pub name: String,
    pub symbol: String,
    pub total_supply: U256,
    pub launch_time: U256,
    pub launch_fee: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPurchaseEvent {
    pub token: Address,
    pub account: Address,
    pub price: U256,
    pub amount: U256,
    pub cost: U256,
    pub fee: U256,
    pub offers: U256,
    pub funds: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenSaleEvent {
    pub token: Address,
    pub account: Address,
    pub price: U256,
    pub amount: U256,
    pub cost: U256,
    pub fee: U256,
    pub offers: U256,
    pub funds: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityAddedEvent {
    pub base: Address,
    pub offers: U256,
    pub quote: Address,
    pub funds: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawTokenManagerEvent {
    pub signature: Option<B256>,
    pub address: Address,
    pub topics: Vec<B256>,
    pub data: alloy::primitives::Bytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventBlockRange {
    pub from_block: u64,
    pub to_block: u64,
}

impl EventBlockRange {
    pub fn chunked(
        from_block: u64,
        to_block: u64,
        chunk_size: u64,
    ) -> crate::Result<Vec<EventBlockRange>> {
        if from_block > to_block {
            return Err(crate::SdkError::InvalidBlockRange {
                from_block,
                to_block,
            });
        }
        if chunk_size == 0 {
            return Err(crate::SdkError::InvalidBlockChunkSize(chunk_size));
        }

        let mut ranges = Vec::new();
        let mut current_from = from_block;
        while current_from <= to_block {
            let current_to = current_from.saturating_add(chunk_size - 1).min(to_block);
            ranges.push(EventBlockRange {
                from_block: current_from,
                to_block: current_to,
            });
            if current_to == u64::MAX {
                break;
            }
            current_from = current_to + 1;
        }

        Ok(ranges)
    }
}

impl From<crate::contracts::TokenManager2::TokenCreate> for TokenCreateEvent {
    fn from(event: crate::contracts::TokenManager2::TokenCreate) -> Self {
        Self {
            creator: event.creator,
            token: event.token,
            request_id: event.requestId,
            name: event.name,
            symbol: event.symbol,
            total_supply: event.totalSupply,
            launch_time: event.launchTime,
            launch_fee: event.launchFee,
        }
    }
}

impl From<crate::contracts::TokenManager2::TokenPurchase> for TokenPurchaseEvent {
    fn from(event: crate::contracts::TokenManager2::TokenPurchase) -> Self {
        Self {
            token: event.token,
            account: event.account,
            price: event.price,
            amount: event.amount,
            cost: event.cost,
            fee: event.fee,
            offers: event.offers,
            funds: event.funds,
        }
    }
}

impl From<crate::contracts::TokenManager2::TokenSale> for TokenSaleEvent {
    fn from(event: crate::contracts::TokenManager2::TokenSale) -> Self {
        Self {
            token: event.token,
            account: event.account,
            price: event.price,
            amount: event.amount,
            cost: event.cost,
            fee: event.fee,
            offers: event.offers,
            funds: event.funds,
        }
    }
}

impl From<crate::contracts::TokenManager2::LiquidityAdded> for LiquidityAddedEvent {
    fn from(event: crate::contracts::TokenManager2::LiquidityAdded) -> Self {
        Self {
            base: event.base,
            offers: event.offers,
            quote: event.quote,
            funds: event.funds,
        }
    }
}

impl From<TokenCreateEvent> for TokenManagerEvent {
    fn from(event: TokenCreateEvent) -> Self {
        Self::TokenCreate(event)
    }
}

impl From<TokenPurchaseEvent> for TokenManagerEvent {
    fn from(event: TokenPurchaseEvent) -> Self {
        Self::TokenPurchase(event)
    }
}

impl From<TokenSaleEvent> for TokenManagerEvent {
    fn from(event: TokenSaleEvent) -> Self {
        Self::TokenSale(event)
    }
}

impl From<LiquidityAddedEvent> for TokenManagerEvent {
    fn from(event: LiquidityAddedEvent) -> Self {
        Self::LiquidityAdded(event)
    }
}

impl TokenEventMetadata {
    pub(crate) fn from_log(log: &alloy::rpc::types::eth::Log) -> Self {
        Self {
            address: log.address(),
            block_hash: log.block_hash,
            block_number: log.block_number,
            block_timestamp: log.block_timestamp,
            transaction_hash: log.transaction_hash,
            transaction_index: log.transaction_index,
            log_index: log.log_index,
            is_removed: log.removed,
        }
    }
}

impl RawTokenManagerEvent {
    pub(crate) fn from_log(log: &alloy::rpc::types::eth::Log, signature: Option<B256>) -> Self {
        Self {
            signature,
            address: log.address(),
            topics: log.topics().to_vec(),
            data: log.data().data.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EventBlockRange;

    #[test]
    fn chunks_inclusive_block_ranges() {
        let ranges = EventBlockRange::chunked(10, 15, 2).expect("valid range");

        assert_eq!(
            ranges,
            vec![
                EventBlockRange {
                    from_block: 10,
                    to_block: 11,
                },
                EventBlockRange {
                    from_block: 12,
                    to_block: 13,
                },
                EventBlockRange {
                    from_block: 14,
                    to_block: 15,
                },
            ]
        );
    }

    #[test]
    fn rejects_invalid_block_range() {
        let error = EventBlockRange::chunked(20, 10, 10).expect_err("invalid range");

        assert!(matches!(
            error,
            crate::SdkError::InvalidBlockRange {
                from_block: 20,
                to_block: 10,
            }
        ));
    }

    #[test]
    fn rejects_zero_chunk_size() {
        let error = EventBlockRange::chunked(10, 20, 0).expect_err("invalid chunk size");

        assert!(matches!(error, crate::SdkError::InvalidBlockChunkSize(0)));
    }
}
