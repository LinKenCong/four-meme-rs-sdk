//! Request and response models used by REST, trading, events, and EIP-8004 helpers.

use std::collections::BTreeMap;

use alloy::primitives::{Address, B256, Bytes, U256};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::utils::{parse_address, validate_https_url, validate_https_url_host};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiCode {
    String(String),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Bool(bool),
}

impl ApiCode {
    pub fn is_success(&self) -> bool {
        match self {
            Self::String(code) => code == "0",
            Self::Signed(code) => *code == 0,
            Self::Unsigned(code) => *code == 0,
            Self::Float(code) => code.abs() < f64::EPSILON,
            Self::Bool(_) => false,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Self::String(code) => code.clone(),
            Self::Signed(code) => code.to_string(),
            Self::Unsigned(code) => code.to_string(),
            Self::Float(code) => code.to_string(),
            Self::Bool(code) => code.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CompatibilityFields {
    #[serde(flatten)]
    fields: BTreeMap<String, Value>,
}

impl CompatibilityFields {
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.fields.keys().map(String::as_str)
    }

    pub fn string(&self, key: &str) -> Option<String> {
        self.fields.get(key).and_then(value_to_string)
    }

    pub fn number(&self, key: &str) -> Option<f64> {
        self.fields.get(key).and_then(value_to_f64)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiEnvelope<T> {
    pub code: ApiCode,
    pub msg: Option<String>,
    pub message: Option<String>,
    #[serde(default = "empty_api_data")]
    pub data: Option<T>,
}

fn empty_api_data<T>() -> Option<T> {
    None
}

impl<T> ApiEnvelope<T> {
    pub fn is_success(&self) -> bool {
        self.code.is_success()
    }

    pub fn code_string(&self) -> String {
        self.code.as_string()
    }

    pub fn message_text(&self) -> String {
        self.msg
            .as_deref()
            .or(self.message.as_deref())
            .unwrap_or("request failed")
            .to_string()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicConfig {
    pub raised_tokens: Vec<RaisedToken>,
    #[serde(flatten)]
    pub extra: CompatibilityFields,
}

impl PublicConfig {
    pub fn len(&self) -> usize {
        self.raised_tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raised_tokens.is_empty()
    }

    pub fn raised_tokens(&self) -> &[RaisedToken] {
        &self.raised_tokens
    }
}

impl From<Vec<RaisedToken>> for PublicConfig {
    fn from(raised_tokens: Vec<RaisedToken>) -> Self {
        Self {
            raised_tokens,
            extra: CompatibilityFields::default(),
        }
    }
}

impl<'de> Deserialize<'de> for PublicConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match PublicConfigWire::deserialize(deserializer)? {
            PublicConfigWire::List(raised_tokens) => raised_tokens.into(),
            PublicConfigWire::Object(response) => Self {
                raised_tokens: response.raised_tokens,
                extra: response.extra,
            },
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PublicConfigWire {
    List(Vec<RaisedToken>),
    Object(PublicConfigObject),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublicConfigObject {
    #[serde(default, alias = "raisedToken", alias = "raisedTokenList")]
    raised_tokens: Vec<RaisedToken>,
    #[serde(flatten)]
    extra: CompatibilityFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaisedToken {
    pub symbol: String,
    #[serde(default)]
    pub symbol_address: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub total_amount: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub total_b_amount: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub sale_rate: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(flatten)]
    pub extra: CompatibilityFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenDetail {
    #[serde(default, alias = "address")]
    pub token_address: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub short_name: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub desc: Option<String>,
    #[serde(default, alias = "imgUrl", alias = "iconUrl", alias = "imageUrl")]
    pub image_url: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub version: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub launch_time: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub market_cap: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub price: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub volume_24h: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub holders: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub progress: Option<String>,
    #[serde(default)]
    pub raised_token: Option<RaisedToken>,
    #[serde(flatten)]
    pub extra: CompatibilityFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenSummary {
    #[serde(default, alias = "address")]
    pub token_address: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub short_name: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub desc: Option<String>,
    #[serde(default, alias = "imgUrl", alias = "iconUrl", alias = "imageUrl")]
    pub image_url: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub version: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub launch_time: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub market_cap: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub price: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub volume_24h: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub holders: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub progress: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub rank: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub rank_change: Option<String>,
    #[serde(flatten)]
    pub extra: CompatibilityFields,
}

pub type TokenSearchResponse = TokenListResponse<TokenSummary>;
pub type TokenRankingEntry = TokenSummary;
pub type TokenRankingResponse = TokenListResponse<TokenRankingEntry>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenListResponse<T> {
    pub list: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(flatten)]
    pub extra: CompatibilityFields,
}

impl<'de, T> Deserialize<'de> for TokenListResponse<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(
            match TokenListResponseWire::<T>::deserialize(deserializer)? {
                TokenListResponseWire::List(list) => Self::from_list(list),
                TokenListResponseWire::Object(response) => Self::from_object(response),
            },
        )
    }
}

impl<T> TokenListResponse<T> {
    fn from_list(list: Vec<T>) -> Self {
        Self {
            list,
            total: None,
            page_index: None,
            page_size: None,
            extra: CompatibilityFields::default(),
        }
    }

    fn from_object(response: TokenListResponseObject<T>) -> Self {
        Self {
            list: response.list,
            total: response.total,
            page_index: response.page_index,
            page_size: response.page_size,
            extra: response.extra,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TokenListResponseWire<T> {
    List(Vec<T>),
    Object(TokenListResponseObject<T>),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenListResponseObject<T> {
    #[serde(alias = "items", alias = "records", alias = "rows", alias = "tokens")]
    list: Vec<T>,
    #[serde(default)]
    total: Option<u64>,
    #[serde(default)]
    page_index: Option<u32>,
    #[serde(default)]
    page_size: Option<u32>,
    #[serde(flatten)]
    extra: CompatibilityFields,
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

impl Serialize for TokenLabel {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_api_str())
    }
}

impl<'de> Deserialize<'de> for TokenLabel {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_from(value.as_str()).map_err(serde::de::Error::custom)
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
        let total = u32::from(self.burn_rate)
            + u32::from(self.divide_rate)
            + u32::from(self.liquidity_rate)
            + u32::from(self.recipient_rate);
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
        match (self.recipient_rate, &self.recipient_address) {
            (0, None) => Ok(()),
            (0, Some(address)) if address.trim().is_empty() => Ok(()),
            (_, Some(address)) => parse_address(address).map(|_| ()),
            (_, None) => Err(crate::SdkError::MissingField(
                "token_tax_info.recipient_address",
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenRequest {
    pub name: String,
    pub short_name: String,
    pub desc: String,
    pub label: TokenLabel,
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
    File {
        file_name: String,
        bytes: Vec<u8>,
        content_type: Option<String>,
    },
    Url(String),
}

impl CreateTokenRequest {
    pub fn validate(&self) -> crate::Result<()> {
        validate_required_text("name", &self.name)?;
        validate_required_text("short_name", &self.short_name)?;
        validate_required_text("desc", &self.desc)?;
        self.image.validate()?;
        validate_create_token_links(self)?;
        if let Some(tax) = &self.token_tax_info {
            tax.validate()?;
        }
        Ok(())
    }
}

impl CreateTokenImage {
    pub fn file(file_name: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self::File {
            file_name: file_name.into(),
            bytes: bytes.into(),
            content_type: None,
        }
    }

    pub fn file_with_content_type(
        file_name: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
        content_type: impl Into<String>,
    ) -> Self {
        Self::File {
            file_name: file_name.into(),
            bytes: bytes.into(),
            content_type: Some(content_type.into()),
        }
    }

    pub fn validate(&self) -> crate::Result<()> {
        match self {
            Self::File {
                file_name,
                bytes,
                content_type,
            } => validate_image_file(file_name, bytes, content_type.as_deref()),
            Self::Url(url) => validate_https_url("image", url),
        }
    }
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

fn validate_required_text(field: &'static str, value: &str) -> crate::Result<()> {
    if value.trim().is_empty() {
        return Err(crate::SdkError::MissingField(field));
    }
    Ok(())
}

fn validate_create_token_links(request: &CreateTokenRequest) -> crate::Result<()> {
    validate_optional_url("web_url", request.web_url.as_deref())?;
    validate_optional_social_url(
        "twitter_url",
        request.twitter_url.as_deref(),
        &["twitter.com", "x.com"],
    )?;
    validate_optional_social_url(
        "telegram_url",
        request.telegram_url.as_deref(),
        &["t.me", "telegram.me", "telegram.org"],
    )
}

fn validate_optional_url(field: &'static str, value: Option<&str>) -> crate::Result<()> {
    let Some(value) = required_optional_text(field, value)? else {
        return Ok(());
    };
    validate_https_url(field, value)
}

fn validate_optional_social_url(
    field: &'static str,
    value: Option<&str>,
    allowed_hosts: &[&str],
) -> crate::Result<()> {
    let Some(value) = required_optional_text(field, value)? else {
        return Ok(());
    };
    validate_https_url_host(field, value, allowed_hosts)
}

fn required_optional_text<'a>(
    field: &'static str,
    value: Option<&'a str>,
) -> crate::Result<Option<&'a str>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.trim().is_empty() {
        return Err(crate::SdkError::MissingField(field));
    }
    Ok(Some(value))
}

fn validate_image_file(
    file_name: &str,
    bytes: &[u8],
    content_type: Option<&str>,
) -> crate::Result<()> {
    let normalized_name = file_name.trim().to_ascii_lowercase();
    if normalized_name.is_empty() {
        return Err(crate::SdkError::MissingField("image.file_name"));
    }
    if bytes.is_empty() {
        return Err(crate::SdkError::InvalidTokenImage(
            "file bytes are empty".to_string(),
        ));
    }
    let extension_type = match normalized_name.rsplit_once('.') {
        Some((_, "png")) => "image/png",
        Some((_, "jpg" | "jpeg")) => "image/jpeg",
        Some((_, "gif")) => "image/gif",
        Some((_, "webp")) => "image/webp",
        _ => {
            return Err(crate::SdkError::InvalidTokenImage(
                "file extension must be png, jpg, jpeg, gif, or webp".to_string(),
            ));
        }
    };
    if let Some(content_type) = content_type
        && content_type != extension_type
    {
        return Err(crate::SdkError::InvalidTokenImage(format!(
            "content type `{content_type}` does not match `{extension_type}`"
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenApiOutput {
    pub create_arg: String,
    pub signature: String,
    pub creation_fee_wei: String,
    #[serde(default)]
    pub calldata: String,
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

fn deserialize_optional_string<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<Value>::deserialize(deserializer)
        .map(|value| value.and_then(|item| value_to_string(&item)))
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Array(_) | Value::Object(_) => None,
    }
}

fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(value) => value.as_f64(),
        Value::String(value) => value.parse().ok(),
        Value::Bool(_) | Value::Array(_) | Value::Object(_) | Value::Null => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ApiEnvelope, CreateTokenImage, CreateTokenRequest, PublicConfig, TokenDetail, TokenLabel,
        TokenRankingResponse, TokenSearchResponse, TokenTaxInfo,
    };

    fn valid_create_token_request() -> CreateTokenRequest {
        CreateTokenRequest {
            name: "Example Token".to_string(),
            short_name: "EXM".to_string(),
            desc: "Example token description".to_string(),
            label: TokenLabel::Meme,
            image: CreateTokenImage::Url("https://example.com/token.png".to_string()),
            web_url: Some("https://example.com".to_string()),
            twitter_url: Some("https://x.com/example".to_string()),
            telegram_url: Some("https://t.me/example".to_string()),
            pre_sale: String::new(),
            fee_plan: false,
            token_tax_info: None,
        }
    }

    #[test]
    fn serializes_token_label_as_api_value() {
        let payload = json!({ "label": TokenLabel::DeSci });

        assert_eq!(payload["label"], "De-Sci");
    }

    #[test]
    fn validates_create_token_url_fields() {
        let mut request = valid_create_token_request();
        request.twitter_url = Some("https://example.com/not-twitter".to_string());

        assert!(request.validate().is_err());
    }

    #[test]
    fn validates_image_metadata() {
        let mut request = valid_create_token_request();
        request.image =
            CreateTokenImage::file_with_content_type("token.png", vec![1], "image/jpeg");

        assert!(request.validate().is_err());
    }

    #[test]
    fn validates_tax_recipient_address() {
        let tax = TokenTaxInfo {
            fee_rate: 5,
            burn_rate: 25,
            divide_rate: 25,
            liquidity_rate: 25,
            recipient_rate: 25,
            recipient_address: Some("not-an-address".to_string()),
            min_sharing: 0,
        };

        assert!(tax.validate().is_err());
    }

    #[test]
    fn parses_public_config_array_fixture() {
        let envelope: ApiEnvelope<PublicConfig> = serde_json::from_value(json!({
            "code": "0",
            "msg": "ok",
            "message": null,
            "data": [
                {
                    "symbol": "BNB",
                    "symbolAddress": "0x0000000000000000000000000000000000000001",
                    "totalAmount": 1000000000,
                    "totalBAmount": "24",
                    "saleRate": 0.8,
                    "status": "PUBLISH",
                    "reserveRate": "0"
                }
            ]
        }))
        .expect("public config fixture should parse");

        assert!(envelope.is_success());
        let config = envelope.data.expect("success fixture has data");
        assert_eq!(config.len(), 1);
        let raised_token = &config.raised_tokens()[0];
        assert_eq!(raised_token.symbol, "BNB");
        assert_eq!(raised_token.total_amount.as_deref(), Some("1000000000"));
        assert_eq!(raised_token.sale_rate.as_deref(), Some("0.8"));
        assert_eq!(
            raised_token.extra.string("reserveRate").as_deref(),
            Some("0")
        );
    }

    #[test]
    fn parses_token_detail_fixture_with_compatibility_fields() {
        let envelope: ApiEnvelope<TokenDetail> = serde_json::from_value(json!({
            "code": 0,
            "msg": "success",
            "message": null,
            "data": {
                "address": "0x1111111111111111111111111111111111111111",
                "name": "Example Meme",
                "shortName": "EXM",
                "imgUrl": "https://example.invalid/exm.png",
                "marketCap": 12345.67,
                "holders": "88",
                "customField": "kept"
            }
        }))
        .expect("token detail fixture should parse");

        assert!(envelope.is_success());
        let detail = envelope.data.expect("success fixture has data");
        assert_eq!(
            detail.token_address.as_deref(),
            Some("0x1111111111111111111111111111111111111111")
        );
        assert_eq!(
            detail.image_url.as_deref(),
            Some("https://example.invalid/exm.png")
        );
        assert_eq!(detail.market_cap.as_deref(), Some("12345.67"));
        assert_eq!(detail.extra.string("customField").as_deref(), Some("kept"));
    }

    #[test]
    fn parses_token_search_fixture() {
        let envelope: ApiEnvelope<TokenSearchResponse> = serde_json::from_value(json!({
            "code": "0",
            "msg": "success",
            "message": null,
            "data": {
                "list": [
                    {
                        "tokenAddress": "0x2222222222222222222222222222222222222222",
                        "shortName": "SEA",
                        "price": "0.0001",
                        "volume24h": 42,
                        "unknownSearchField": true
                    }
                ],
                "total": 1,
                "pageIndex": 1,
                "pageSize": 30,
                "hasNext": false
            }
        }))
        .expect("token search fixture should parse");

        let response = envelope.data.expect("success fixture has data");
        assert_eq!(response.total, Some(1));
        assert_eq!(response.page_size, Some(30));
        assert_eq!(response.list[0].short_name.as_deref(), Some("SEA"));
        assert_eq!(response.list[0].volume_24h.as_deref(), Some("42"));
        assert!(response.extra.contains_key("hasNext"));
        let token = &response.list[0];
        assert_eq!(
            token.extra.string("unknownSearchField").as_deref(),
            Some("true")
        );
    }

    #[test]
    fn parses_token_ranking_fixture_from_array_data() {
        let envelope: ApiEnvelope<TokenRankingResponse> = serde_json::from_value(json!({
            "code": "0",
            "msg": "success",
            "message": null,
            "data": [
                {
                    "tokenAddress": "0x3333333333333333333333333333333333333333",
                    "shortName": "RNK",
                    "rank": 1,
                    "rankChange": "-2",
                    "marketCap": "9999"
                }
            ]
        }))
        .expect("token ranking fixture should parse");

        let response = envelope.data.expect("success fixture has data");
        assert_eq!(response.list.len(), 1);
        assert_eq!(response.list[0].rank.as_deref(), Some("1"));
        assert_eq!(response.list[0].rank_change.as_deref(), Some("-2"));
        assert_eq!(response.list[0].market_cap.as_deref(), Some("9999"));
    }

    #[test]
    fn parses_error_envelope_without_data() {
        let envelope: ApiEnvelope<PublicConfig> = serde_json::from_value(json!({
            "code": "40001",
            "msg": "validation failed"
        }))
        .expect("error envelope without data should parse");

        assert!(!envelope.is_success());
        assert!(envelope.data.is_none());
        assert_eq!(envelope.message_text(), "validation failed");
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuyMode {
    FixedAmount { amount: U256, max_funds: U256 },
    FixedFunds { funds: U256, min_amount: U256 },
}

impl BuyMode {
    pub fn validate(self) -> crate::Result<()> {
        match self {
            Self::FixedAmount { amount, max_funds } => {
                validate_trade_amount("amount", amount)?;
                validate_trade_amount("max_funds", max_funds)
            }
            Self::FixedFunds { funds, min_amount } => {
                validate_trade_amount("funds", funds)?;
                validate_trade_amount("min_amount", min_amount)
            }
        }
    }

    pub fn quote_inputs(self) -> (U256, U256) {
        match self {
            Self::FixedAmount { amount, .. } => (amount, U256::ZERO),
            Self::FixedFunds { funds, .. } => (U256::ZERO, funds),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeApproval {
    pub token: Address,
    pub spender: Address,
    pub amount: U256,
    #[serde(default)]
    pub calldata: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuyExecutionPlan {
    FixedAmount {
        token: Address,
        value: U256,
        amount: U256,
        max_funds: U256,
        #[serde(default)]
        calldata: Bytes,
    },
    FixedFunds {
        token: Address,
        value: U256,
        funds: U256,
        min_amount: U256,
        #[serde(default)]
        calldata: Bytes,
    },
}

impl BuyExecutionPlan {
    pub fn value(&self) -> U256 {
        match self {
            Self::FixedAmount { value, .. } | Self::FixedFunds { value, .. } => *value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuyPlan {
    pub token: Address,
    pub token_manager: Address,
    pub mode: BuyMode,
    pub quote: BuyQuote,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval: Option<TradeApproval>,
    pub execution: BuyExecutionPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SellExecutionPlan {
    pub token: Address,
    pub value: U256,
    pub amount: U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_funds: Option<U256>,
    #[serde(default)]
    pub calldata: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SellPlan {
    pub token: Address,
    pub token_manager: Address,
    pub quote: SellQuote,
    pub approval: TradeApproval,
    pub execution: SellExecutionPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeApprovalReceipt {
    pub approval: TradeApproval,
    pub receipt: ConfirmedReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeExecutionReceipt {
    pub token: Address,
    pub token_manager: Address,
    pub value: U256,
    pub receipt: ConfirmedReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuyExecutionResult {
    pub plan: BuyPlan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval: Option<TradeApprovalReceipt>,
    pub execution: TradeExecutionReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SellExecutionResult {
    pub plan: SellPlan,
    pub approval: TradeApprovalReceipt,
    pub execution: TradeExecutionReceipt,
}

fn validate_trade_amount(field: &'static str, amount: U256) -> crate::Result<()> {
    if amount == U256::ZERO {
        return Err(crate::SdkError::validation(
            field,
            "must be greater than zero",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum Asset {
    Native,
    Erc20(Address),
}

/// Validated metadata used to build an EIP-8004 agent registration URI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMetadata {
    pub name: String,
    pub image_url: String,
    pub description: String,
}

impl AgentMetadata {
    /// Creates agent metadata after trimming fields and validating the image URL.
    pub fn new(
        name: impl AsRef<str>,
        image_url: impl AsRef<str>,
        description: impl AsRef<str>,
    ) -> crate::Result<Self> {
        let name = require_metadata_field("name", name.as_ref())?;
        let image_url = require_metadata_field("image_url", image_url.as_ref())?;
        let description = normalize_optional_metadata_field(description.as_ref());
        validate_metadata_url(&image_url)?;

        Ok(Self {
            name,
            image_url,
            description,
        })
    }
}

/// Result of an EIP-8004 agent registration transaction.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmedReceipt {
    pub tx_hash: B256,
    pub block_number: Option<u64>,
    pub gas_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRegistration {
    pub tx_hash: B256,
    pub agent_id: U256,
    pub agent_uri: String,
}

fn require_metadata_field(field: &'static str, value: &str) -> crate::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(crate::SdkError::MissingField(field));
    }
    Ok(value.to_string())
}

fn normalize_optional_metadata_field(value: &str) -> String {
    value.trim().to_string()
}

fn validate_metadata_url(value: &str) -> crate::Result<()> {
    let url = url::Url::parse(value).map_err(|_| crate::SdkError::InvalidField {
        field: "image_url",
        reason: "must be an absolute http(s) URL",
    })?;
    if matches!(url.scheme(), "http" | "https") {
        Ok(())
    } else {
        Err(crate::SdkError::InvalidField {
            field: "image_url",
            reason: "must use http or https",
        })
    }
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
mod event_tests {
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
