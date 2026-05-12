use alloy::primitives::{Address, Bytes, U256};
use base64::Engine;

use crate::error::{Result, SdkError};

pub const ZERO_ADDRESS: Address = Address::ZERO;

pub fn parse_address(value: impl AsRef<str>) -> Result<Address> {
    value.as_ref().parse::<Address>().map_err(|_| {
        SdkError::validation(
            "address",
            format!("invalid address `{}`", value.as_ref()),
        )
    })
}

pub fn parse_u256(value: impl AsRef<str>) -> Result<U256> {
    value.as_ref().parse::<U256>().map_err(|_| {
        SdkError::validation("amount", format!("invalid amount `{}`", value.as_ref()))
    })
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

pub fn bnb_to_wei_lossy(amount: f64) -> U256 {
    if amount <= 0.0 {
        return U256::ZERO;
    }
    U256::from((amount * 1_000_000_000_000_000_000f64).round() as u128)
}
