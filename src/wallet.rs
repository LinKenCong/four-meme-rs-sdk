use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;

use crate::error::{Result, SdkError};

pub fn signer_from_private_key(private_key: impl AsRef<str>) -> Result<PrivateKeySigner> {
    let raw = private_key.as_ref().trim();
    let normalized = raw.strip_prefix("0x").unwrap_or(raw);
    normalized
        .parse::<PrivateKeySigner>()
        .map_err(|_| SdkError::InvalidPrivateKey)
}

pub fn assert_signer_address(signer: &PrivateKeySigner, expected: Address) -> Result<()> {
    let actual = signer.address();
    if actual != expected {
        return Err(SdkError::WalletAddressMismatch { expected, actual });
    }
    Ok(())
}
