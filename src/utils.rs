//! Parsing and normalization helpers shared by SDK examples and callers.

use alloy::primitives::{Address, Bytes, U256};
use base64::Engine;
use url::Url;

use crate::error::{Result, SdkError};

pub const ZERO_ADDRESS: Address = Address::ZERO;

pub fn parse_address(value: impl AsRef<str>) -> Result<Address> {
    value
        .as_ref()
        .parse::<Address>()
        .map_err(|_| SdkError::InvalidAddress(value.as_ref().to_string()))
}

pub fn parse_u256(value: impl AsRef<str>) -> Result<U256> {
    value
        .as_ref()
        .parse::<U256>()
        .map_err(|_| SdkError::InvalidAmount(value.as_ref().to_string()))
}

pub fn normalize_hex_or_base64(value: impl AsRef<str>) -> Result<Bytes> {
    let raw = value.as_ref().trim();
    if let Some(stripped) = raw.strip_prefix("0x") {
        return Ok(hex::decode(stripped)?.into());
    }
    if raw.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Ok(hex::decode(raw)?.into());
    }
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(raw)
        .map_err(|error| SdkError::Contract(format!("invalid hex/base64 payload: {error}")))?;
    Ok(decoded.into())
}

pub fn optional_non_zero(address: Address) -> Option<Address> {
    (address != ZERO_ADDRESS).then_some(address)
}

pub fn validate_https_url(field: &'static str, value: &str) -> Result<()> {
    parse_https_url(field, value).map(|_| ())
}

pub fn validate_https_url_host(
    field: &'static str,
    value: &str,
    allowed_hosts: &[&str],
) -> Result<()> {
    let parsed = parse_https_url(field, value)?;
    let host = parsed.host_str().ok_or_else(|| SdkError::InvalidUrlField {
        field,
        value: value.to_string(),
    })?;
    if allowed_hosts.contains(&host) {
        return Ok(());
    }
    Err(SdkError::InvalidUrlField {
        field,
        value: value.to_string(),
    })
}

fn parse_https_url(field: &'static str, value: &str) -> Result<Url> {
    let parsed = Url::parse(value).map_err(|_| SdkError::InvalidUrlField {
        field,
        value: value.to_string(),
    })?;
    if parsed.scheme() != "https" || parsed.host_str().is_none() {
        return Err(SdkError::InvalidUrlField {
            field,
            value: value.to_string(),
        });
    }
    Ok(parsed)
}

pub const WEI_DECIMALS: usize = 18;

/// Parses a BNB decimal string into wei without floating-point rounding.
pub fn parse_bnb_to_wei(value: impl AsRef<str>) -> Result<U256> {
    parse_decimal_units(value, WEI_DECIMALS)
}

/// Parses a non-negative decimal string into the smallest unit for the given precision.
pub fn parse_decimal_units(value: impl AsRef<str>, decimals: usize) -> Result<U256> {
    let original = value.as_ref();
    let trimmed = original.trim();
    if trimmed.is_empty() {
        return Err(SdkError::InvalidAmount(original.to_string()));
    }

    let (whole, fraction) = split_decimal(trimmed, decimals, original)?;
    let scaled = whole
        .bytes()
        .chain(fraction.unwrap_or_default().bytes())
        .try_fold(U256::ZERO, append_decimal_digit)
        .ok_or_else(|| SdkError::InvalidAmount(original.to_string()))?;
    scaled
        .checked_mul(pow10(decimals - fraction.map_or(0, str::len), original)?)
        .ok_or_else(|| SdkError::InvalidAmount(original.to_string()))
}

fn split_decimal<'a>(
    value: &'a str,
    decimals: usize,
    original: &str,
) -> Result<(&'a str, Option<&'a str>)> {
    let (whole, fraction) = value
        .split_once('.')
        .map_or((value, None), |parts| (parts.0, Some(parts.1)));
    if !is_ascii_digits(whole) || fraction.is_some_and(|value| !is_ascii_digits(value)) {
        return Err(SdkError::InvalidAmount(original.to_string()));
    }
    if fraction.is_some_and(|value| value.len() > decimals) {
        return Err(SdkError::InvalidAmount(original.to_string()));
    }
    Ok((whole, fraction))
}

fn append_decimal_digit(value: U256, digit: u8) -> Option<U256> {
    value
        .checked_mul(U256::from(10u8))
        .and_then(|next| next.checked_add(U256::from(digit - b'0')))
}

fn pow10(exponent: usize, original: &str) -> Result<U256> {
    (0..exponent).try_fold(U256::from(1u8), |value, _| {
        value
            .checked_mul(U256::from(10u8))
            .ok_or_else(|| SdkError::InvalidAmount(original.to_string()))
    })
}

fn is_ascii_digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}
