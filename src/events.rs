use alloy::providers::Provider;
use alloy::rpc::types::eth::{Filter, Log};
use alloy::sol_types::SolEvent;

use crate::client::FourMemeSdk;
use crate::contracts::TokenManager2;
use crate::error::{Result, SdkError};
use crate::types::{
    EventBlockRange, LiquidityAddedEvent, RawTokenManagerEvent, TokenCreateEvent, TokenEvent,
    TokenEventMetadata, TokenManagerEvent, TokenPurchaseEvent, TokenSaleEvent,
};

const DEFAULT_EVENT_BLOCK_CHUNK_SIZE: u64 = 2_000;

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
        self.events_with_chunk_size(from_block, to_block, DEFAULT_EVENT_BLOCK_CHUNK_SIZE)
            .await
    }

    pub async fn events_with_chunk_size(
        &self,
        from_block: u64,
        to_block: Option<u64>,
        chunk_size: u64,
    ) -> Result<Vec<TokenEvent>> {
        let to_block = match to_block {
            Some(to_block) => to_block,
            None => self
                .provider
                .get_block_number()
                .await
                .map_err(|error| SdkError::Contract(error.to_string()))?,
        };
        let ranges = EventBlockRange::chunked(from_block, to_block, chunk_size)?;
        let mut events = Vec::new();

        for range in ranges {
            let filter = self.token_manager_event_filter(range.from_block, range.to_block);
            let logs = self
                .provider
                .get_logs(&filter)
                .await
                .map_err(|error| SdkError::Contract(error.to_string()))?;
            events.extend(logs.into_iter().map(decode_token_manager_event));
        }

        Ok(events)
    }

    fn token_manager_event_filter(&self, from_block: u64, to_block: u64) -> Filter {
        Filter::new()
            .address(self.config.addresses.token_manager2)
            .from_block(from_block)
            .to_block(to_block)
            .event_signature(TokenManagerEvent::signature_hashes())
    }
}

fn decode_token_manager_event(log: Log) -> TokenEvent {
    let metadata = TokenEventMetadata::from_log(&log);
    let kind = match log.topic0().copied() {
        Some(TokenManager2::TokenCreate::SIGNATURE_HASH) => {
            decode_known_event::<TokenManager2::TokenCreate, _>(&log, TokenCreateEvent::from)
        }
        Some(TokenManager2::TokenPurchase::SIGNATURE_HASH) => {
            decode_known_event::<TokenManager2::TokenPurchase, _>(&log, TokenPurchaseEvent::from)
        }
        Some(TokenManager2::TokenSale::SIGNATURE_HASH) => {
            decode_known_event::<TokenManager2::TokenSale, _>(&log, TokenSaleEvent::from)
        }
        Some(TokenManager2::LiquidityAdded::SIGNATURE_HASH) => {
            decode_known_event::<TokenManager2::LiquidityAdded, _>(&log, LiquidityAddedEvent::from)
        }
        topic0 => TokenManagerEvent::Raw(RawTokenManagerEvent::from_log(&log, topic0)),
    };

    TokenEvent { metadata, kind }
}

fn decode_known_event<TEvent, TTyped>(
    log: &Log,
    build_typed: impl FnOnce(TEvent) -> TTyped,
) -> TokenManagerEvent
where
    TEvent: SolEvent,
    TTyped: Into<TokenManagerEvent>,
{
    match log.log_decode::<TEvent>() {
        Ok(decoded) => build_typed(decoded.inner.data).into(),
        Err(_) => {
            TokenManagerEvent::Raw(RawTokenManagerEvent::from_log(log, log.topic0().copied()))
        }
    }
}
