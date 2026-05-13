//! Wallet helpers for explicit signer injection.

use alloy::primitives::Address;
use alloy::signers::Signer;
use alloy::signers::local::PrivateKeySigner;

use crate::error::{Result, SdkError};

/// Builds a local signer from a hex-encoded private key string.
///
/// The input is trimmed and may include a `0x` prefix. Parse failures are
/// intentionally collapsed to avoid leaking the provided secret in errors.
pub fn signer_from_private_key(private_key: impl AsRef<str>) -> Result<PrivateKeySigner> {
    let raw = private_key.as_ref().trim();
    let normalized = raw.strip_prefix("0x").unwrap_or(raw);
    normalized
        .parse::<PrivateKeySigner>()
        .map_err(|_| SdkError::InvalidPrivateKey)
}

/// Verifies that a caller-provided signer belongs to the expected wallet.
pub fn assert_signer_address(signer: &impl Signer, expected: Address) -> Result<()> {
    let actual = signer.address();
    if actual != expected {
        return Err(SdkError::WalletAddressMismatch { expected, actual });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_key_parse_errors_do_not_expose_secret() {
        let secret = "invalid-test-secret";
        let malformed = format!("{secret}-not-hex");
        let error = signer_from_private_key(&malformed).expect_err("key must be invalid");

        assert!(!format!("{error}").contains(secret));
        assert!(!format!("{error:?}").contains(secret));
    }

    #[test]
    fn private_key_parser_accepts_prefixed_and_unprefixed_keys() {
        let prefixed = format!("0x{}", "11".repeat(32));
        let unprefixed = "11".repeat(32);

        let prefixed_signer = signer_from_private_key(&prefixed).expect("prefixed key parses");
        let unprefixed_signer = signer_from_private_key(&unprefixed).expect("raw key parses");

        assert_eq!(prefixed_signer.address(), unprefixed_signer.address());
    }
}
