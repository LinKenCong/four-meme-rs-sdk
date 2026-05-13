mod common;

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = common::build_read_only_sdk()?;
    let config = sdk.public_config().await?;

    for token in config.raised_tokens().iter().take(5) {
        println!(
            "symbol={} status={}",
            token.symbol,
            token.status.as_deref().unwrap_or("unknown")
        );
    }

    Ok(())
}
