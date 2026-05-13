mod common;

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = common::build_read_only_sdk()?;
    let events = sdk.recent_events(2_000).await?;

    for event in events.iter().take(10) {
        println!(
            "event={} block={} tx={}",
            event.event_name(),
            event
                .metadata
                .block_number
                .map_or_else(|| "pending".to_string(), |block| block.to_string()),
            event
                .metadata
                .transaction_hash
                .map_or_else(|| "unknown".to_string(), |tx| tx.to_string())
        );
    }

    Ok(())
}
