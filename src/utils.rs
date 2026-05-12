use alloy::primitives::{Address, Bytes, U256};
use base64::Engine;

use crate::error::{Result, SdkError};

pub const ZERO_ADDRESS: Address = Address::ZERO;

pub fn parse_address(value: impl AsRef<str>) -> Result<Address> {
    value.as_ref().parse::<Address>().map_err(|_| {
        SdkError::validation("address", format!("invalid address `{}`", value.as_ref()))
    })
}

pub fn parse_u256(value: impl AsRef<str>) -> Result<U256> {
    value
        .as_ref()
        .parse::<U256>()
        .map_err(|_| SdkError::validation("amount", format!("invalid amount `{}`", value.as_ref())))
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
        .map_err(|error| SdkError::validation("hex_or_base64_payload", error))?;
    Ok(decoded.into())
}

pub fn optional_non_zero(address: Address) -> Option<Address> {
    (address != ZERO_ADDRESS).then_some(address)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bnb_to_wei_preserves_fractional_precision() {
        assert_eq!(parse_bnb_to_wei("1").unwrap(), U256::from(10u128.pow(18)));
        assert_eq!(parse_bnb_to_wei("0.1").unwrap(), U256::from(10u128.pow(17)));
        assert_eq!(
            parse_bnb_to_wei("0.000000000000000001").unwrap(),
            U256::from(1u8)
        );
        assert_eq!(
            parse_bnb_to_wei("123456789.123456789123456789").unwrap(),
            U256::from(123456789123456789123456789u128)
        );
    }

    #[test]
    fn parse_bnb_to_wei_rejects_imprecise_or_invalid_decimals() {
        for value in [
            "",
            " ",
            ".1",
            "1.",
            "-1",
            "+1",
            "1e-18",
            "1,000",
            "1.0000000000000000000",
        ] {
            assert!(
                parse_bnb_to_wei(value).is_err(),
                "{value} should be invalid"
            );
        }
    }

    #[test]
    fn parse_bnb_to_wei_rejects_u256_overflow() {
        let overflowing = format!("{}0", U256::MAX);
        assert!(parse_bnb_to_wei(overflowing).is_err());
    }
}
