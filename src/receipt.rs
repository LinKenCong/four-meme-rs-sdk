//! Transaction receipt confirmation and status validation helpers.

use alloy::network::Ethereum;
use alloy::providers::{PendingTransactionBuilder, PendingTransactionError};
use alloy::rpc::types::TransactionReceipt;

use crate::error::{Result, SdkError};
use crate::types::ConfirmedReceipt;

pub(crate) async fn wait_for_confirmation(
    pending: PendingTransactionBuilder<Ethereum>,
) -> Result<ConfirmedReceipt> {
    let receipt = pending
        .get_receipt()
        .await
        .map_err(|error| pending_error("wait for transaction receipt", error))?;
    confirm_receipt(receipt)
}

pub(crate) fn confirm_receipt(receipt: TransactionReceipt) -> Result<ConfirmedReceipt> {
    if !receipt.status() {
        return Err(SdkError::transaction_failed_with_hash(
            "transaction receipt",
            Some(receipt.transaction_hash),
            format!(
                "transaction reverted in block {:?}; gas used {}",
                receipt.block_number, receipt.gas_used
            ),
        ));
    }

    Ok(ConfirmedReceipt {
        tx_hash: receipt.transaction_hash,
        block_number: receipt.block_number,
        gas_used: receipt.gas_used,
    })
}

fn pending_error(operation: &'static str, error: PendingTransactionError) -> SdkError {
    SdkError::transaction_failed(operation, error)
}

#[cfg(test)]
mod tests {
    use super::confirm_receipt;
    use alloy::consensus::{Eip658Value, Receipt, ReceiptEnvelope};
    use alloy::primitives::{Address, B256};
    use alloy::rpc::types::TransactionReceipt;

    #[test]
    fn confirms_successful_receipt() {
        let receipt = build_receipt(true);

        let confirmed = confirm_receipt(receipt).expect("successful receipt should confirm");

        assert_eq!(confirmed.tx_hash, tx_hash());
        assert_eq!(confirmed.block_number, Some(42));
        assert_eq!(confirmed.gas_used, 21_000);
    }

    #[test]
    fn rejects_reverted_receipt() {
        let receipt = build_receipt(false);

        let error = confirm_receipt(receipt).expect_err("reverted receipt should fail");

        assert!(error.to_string().contains("transaction reverted"));
    }

    fn build_receipt(is_successful: bool) -> TransactionReceipt {
        let inner = ReceiptEnvelope::Legacy(
            Receipt {
                status: Eip658Value::Eip658(is_successful),
                cumulative_gas_used: 21_000,
                logs: Vec::new(),
            }
            .with_bloom(),
        );

        TransactionReceipt {
            inner,
            transaction_hash: tx_hash(),
            transaction_index: Some(0),
            block_hash: Some(B256::with_last_byte(2)),
            block_number: Some(42),
            gas_used: 21_000,
            effective_gas_price: 1,
            blob_gas_used: None,
            blob_gas_price: None,
            from: Address::with_last_byte(1),
            to: Some(Address::with_last_byte(2)),
            contract_address: None,
        }
    }

    fn tx_hash() -> B256 {
        B256::with_last_byte(1)
    }
}
