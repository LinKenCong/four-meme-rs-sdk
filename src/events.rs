use alloy::providers::Provider;
use alloy::rpc::types::eth::Filter;
use alloy::sol_types::SolEvent;
use serde_json::{Value, json};

use crate::client::FourMemeSdk;
use crate::error::{Result, SdkError};
use crate::types::TokenEvent;

impl FourMemeSdk {
    pub async fn recent_events(&self, block_count: u64) -> Result<Vec<TokenEvent>> {
        let latest = self
            .provider
            .get_block_number()
            .await
            .map_err(|error| SdkError::Contract(error.to_string()))?;
        let from = latest.saturating_sub(block_count);
        self.events(from, Some(latest)).await
    }

    pub async fn events(&self, from_block: u64, to_block: Option<u64>) -> Result<Vec<TokenEvent>> {
        let mut filter = Filter::new()
            .address(self.config.addresses.token_manager2)
            .from_block(from_block)
            .event_signature(vec![
                crate::contracts::TokenManager2::TokenCreate::SIGNATURE_HASH,
                crate::contracts::TokenManager2::TokenPurchase::SIGNATURE_HASH,
                crate::contracts::TokenManager2::TokenSale::SIGNATURE_HASH,
                crate::contracts::TokenManager2::LiquidityAdded::SIGNATURE_HASH,
            ]);
        if let Some(to_block) = to_block {
            filter = filter.to_block(to_block);
        }
        let logs = self
            .provider
            .get_logs(&filter)
            .await
            .map_err(|error| SdkError::Contract(error.to_string()))?;
        Ok(logs
            .into_iter()
            .map(|log| {
                let topic0 = log.topic0().copied();
                let event_name = match topic0 {
                    Some(crate::contracts::TokenManager2::TokenCreate::SIGNATURE_HASH) => {
                        "TokenCreate"
                    }
                    Some(crate::contracts::TokenManager2::TokenPurchase::SIGNATURE_HASH) => {
                        "TokenPurchase"
                    }
                    Some(crate::contracts::TokenManager2::TokenSale::SIGNATURE_HASH) => "TokenSale",
                    Some(crate::contracts::TokenManager2::LiquidityAdded::SIGNATURE_HASH) => {
                        "LiquidityAdded"
                    }
                    _ => "Unknown",
                };
                TokenEvent {
                    event_name: event_name.to_string(),
                    block_number: log.block_number.unwrap_or_default(),
                    transaction_hash: log.transaction_hash.unwrap_or_default(),
                    args: serde_json::to_value(&log).unwrap_or(Value::Null),
                }
            })
            .collect())
    }
}

#[allow(dead_code)]
fn event_placeholder(hash: alloy::primitives::B256) -> TokenEvent {
    TokenEvent {
        event_name: "TokenCreate".to_string(),
        block_number: 0,
        transaction_hash: hash,
        args: json!({}),
    }
}
