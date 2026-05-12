use alloy::primitives::{Address, B256};
use reqwest::StatusCode;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SdkError>;

/// Stable SDK error taxonomy for callers that need actionable diagnostics.
#[derive(Debug, Error)]
pub enum SdkError {
    #[error("validation failed for `{field}`: {message}")]
    Validation {
        field: &'static str,
        message: String,
    },
    #[error("configuration field `{field}` is invalid: {message}")]
    Config {
        field: &'static str,
        message: String,
    },
    #[error("http request failed during {operation}: {message}")]
    Http {
        operation: &'static str,
        status: Option<StatusCode>,
        message: String,
        #[source]
        source: Option<reqwest::Error>,
    },
    #[error("Four.meme API rejected request with code {code}: {message}; context: {context}")]
    RestBusiness {
        code: String,
        message: String,
        context: RedactedContext,
    },
    #[error("rpc/provider call failed during {operation}: {message}")]
    RpcProvider {
        operation: &'static str,
        message: String,
    },
    #[error("transaction failed during {operation}: {message}")]
    TransactionFailed {
        operation: &'static str,
        tx_hash: Option<B256>,
        message: String,
    },
    #[error("signing failed during {operation}: {message}")]
    Signing {
        operation: &'static str,
        message: String,
    },
    #[error("serialization failed during {operation}: {message}")]
    Serialization {
        operation: &'static str,
        message: String,
    },
    #[error("io failed during {operation}: {message}")]
    Io {
        operation: &'static str,
        message: String,
        #[source]
        source: Option<std::io::Error>,
    },

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
    #[error("url parse failed: {0}")]
    Url(url::ParseError),
    #[error("json failed: {0}")]
    Json(serde_json::Error),
    #[error("hex decode failed: {0}")]
    Hex(hex::FromHexError),
    #[error("wallet address {actual} does not match expected address {expected}")]
    WalletAddressMismatch { expected: Address, actual: Address },
}

impl SdkError {
    pub fn validation(field: &'static str, message: impl std::fmt::Display) -> Self {
        Self::Validation {
            field,
            message: redact_secret_fragments(&message.to_string()),
        }
    }

    pub fn config(field: &'static str, message: impl std::fmt::Display) -> Self {
        Self::Config {
            field,
            message: redact_secret_fragments(&message.to_string()),
        }
    }

    pub fn http(operation: &'static str, error: reqwest::Error) -> Self {
        Self::Http {
            operation,
            status: error.status(),
            message: redact_secret_fragments(&error.to_string()),
            source: Some(error),
        }
    }

    pub fn rest_business(
        code: impl std::fmt::Display,
        message: impl std::fmt::Display,
        context: RedactedContext,
    ) -> Self {
        Self::RestBusiness {
            code: redact_secret_fragments(&code.to_string()),
            message: redact_secret_fragments(&message.to_string()),
            context,
        }
    }

    pub fn rpc_provider(operation: &'static str, error: impl std::fmt::Display) -> Self {
        Self::RpcProvider {
            operation,
            message: redact_secret_fragments(&error.to_string()),
        }
    }

    pub fn transaction_failed(operation: &'static str, error: impl std::fmt::Display) -> Self {
        Self::transaction_failed_with_hash(operation, None, error)
    }

    pub fn transaction_failed_with_hash(
        operation: &'static str,
        tx_hash: Option<B256>,
        error: impl std::fmt::Display,
    ) -> Self {
        Self::TransactionFailed {
            operation,
            tx_hash,
            message: redact_secret_fragments(&error.to_string()),
        }
    }

    pub fn signing(operation: &'static str, error: impl std::fmt::Display) -> Self {
        Self::Signing {
            operation,
            message: redact_secret_fragments(&error.to_string()),
        }
    }

    pub fn serialization(operation: &'static str, error: impl std::fmt::Display) -> Self {
        Self::Serialization {
            operation,
            message: redact_secret_fragments(&error.to_string()),
        }
    }

    pub fn io(operation: &'static str, error: std::io::Error) -> Self {
        Self::Io {
            operation,
            message: redact_secret_fragments(&error.to_string()),
            source: Some(error),
        }
    }
}

impl From<reqwest::Error> for SdkError {
    fn from(error: reqwest::Error) -> Self {
        Self::http("http request", error)
    }
}

impl From<url::ParseError> for SdkError {
    fn from(error: url::ParseError) -> Self {
        Self::config("url", error)
    }
}

impl From<serde_json::Error> for SdkError {
    fn from(error: serde_json::Error) -> Self {
        Self::serialization("json", error)
    }
}

impl From<hex::FromHexError> for SdkError {
    fn from(error: hex::FromHexError) -> Self {
        Self::validation("hex_payload", error)
    }
}

impl From<std::io::Error> for SdkError {
    fn from(error: std::io::Error) -> Self {
        Self::io("io", error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactedContext {
    entries: Vec<RedactedContextEntry>,
}

impl RedactedContext {
    pub fn new<I, K, V>(entries: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: ToString,
    {
        Self {
            entries: entries
                .into_iter()
                .map(|(key, value)| RedactedContextEntry::new(key, value))
                .collect(),
        }
    }

    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn entries(&self) -> &[RedactedContextEntry] {
        &self.entries
    }
}

impl std::fmt::Display for RedactedContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.entries.is_empty() {
            return formatter.write_str("none");
        }
        let rendered = self
            .entries
            .iter()
            .map(|entry| format!("{}={}", entry.key, entry.value))
            .collect::<Vec<_>>()
            .join(", ");
        formatter.write_str(&rendered)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactedContextEntry {
    key: String,
    value: String,
}

impl RedactedContextEntry {
    fn new(key: impl Into<String>, value: impl ToString) -> Self {
        let key = key.into();
        let value = redact_context_value(&key, &value.to_string());
        Self { key, value }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

fn redact_context_value(key: &str, value: &str) -> String {
    if is_secret_key(key) {
        return "[redacted]".to_string();
    }
    redact_secret_fragments(value)
}

fn redact_secret_fragments(value: &str) -> String {
    let value = redact_json_secret_fields(value);
    redact_secret_text(&value)
}

fn redact_json_secret_fields(value: &str) -> String {
    let Ok(mut json) = serde_json::from_str::<serde_json::Value>(value) else {
        return value.to_string();
    };
    redact_json_value(&mut json);
    json.to_string()
}

fn redact_json_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            for (key, value) in fields {
                if is_secret_key(key) {
                    *value = serde_json::Value::String("[redacted]".to_string());
                } else {
                    redact_json_value(value);
                }
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                redact_json_value(value);
            }
        }
        serde_json::Value::String(value) => {
            *value = redact_secret_text(value);
        }
        _ => {}
    }
}

fn is_secret_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    normalized.contains("access")
        || normalized.contains("auth")
        || normalized.contains("private")
        || normalized.contains("secret")
        || normalized.contains("signature")
        || normalized.contains("password")
        || normalized.contains("api_key")
        || normalized.contains("apikey")
}

fn redact_secret_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(redact_secret_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_secret_word(word: &str) -> String {
    let trimmed = word.trim_matches(|ch: char| !ch.is_ascii_hexdigit() && ch != 'x' && ch != 'X');
    let stripped = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    if stripped.len() == 64 && stripped.chars().all(|ch| ch.is_ascii_hexdigit()) {
        word.replace(trimmed, "[redacted]")
    } else {
        word.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_redacts_private_key_shaped_values() {
        let error = SdkError::validation(
            "private_key",
            "invalid 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );

        let rendered = error.to_string();
        assert!(rendered.contains("[redacted]"));
        assert!(
            !rendered.contains("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
    }

    #[test]
    fn rest_business_context_redacts_secret_fields() {
        let context = RedactedContext::new([
            ("path", "/private/token/create"),
            ("meme-web-access", "test-access-token"),
            (
                "signature",
                "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
        ]);
        let error = SdkError::rest_business("4001", "business rejected", context);

        let rendered = error.to_string();
        assert!(rendered.contains("path=/private/token/create"));
        assert!(rendered.contains("meme-web-access=[redacted]"));
        assert!(rendered.contains("signature=[redacted]"));
        assert!(!rendered.contains("test-access-token"));
        assert!(
            !rendered.contains("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        );
    }

    #[test]
    fn rest_business_context_redacts_secret_json_fields() {
        let context = RedactedContext::new([(
            "response_body",
            r#"{"code":"1","data":{"symbol":"TEST"},"signature":"0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"}"#,
        )]);
        let error = SdkError::rest_business("1", "business rejected", context);

        let rendered = error.to_string();
        assert!(rendered.contains("[redacted]"));
        assert!(rendered.contains("TEST"));
        assert!(
            !rendered.contains("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc")
        );
    }

    #[test]
    fn stable_error_classes_render_actionable_messages() {
        let cases = [
            SdkError::config("chain_id", "unsupported chain id 97"),
            SdkError::http("token detail", http_error()),
            SdkError::rpc_provider("quote buy", "provider timeout"),
            SdkError::transaction_failed("buy", "receipt status 0"),
            SdkError::signing("login", "signer unavailable"),
        ];

        for error in cases {
            let rendered = error.to_string();
            assert!(!rendered.is_empty());
            assert!(
                !rendered
                    .contains("0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc")
            );
        }
    }

    fn http_error() -> reqwest::Error {
        reqwest::Client::new()
            .get("http://")
            .build()
            .expect_err("invalid URL should create a request builder error")
    }
}
