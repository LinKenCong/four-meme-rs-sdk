mod common;

use alloy::primitives::U256;

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let Some(token) = common::example_token_address()? else {
        common::skip_missing_address("quotes", common::EXAMPLE_TOKEN_ADDRESS_ENV);
        return Ok(());
    };
    let sdk = common::build_read_only_sdk()?;
    let token_info = sdk.get_token_info(token).await?;

    println!("token manager: {}", token_info.token_manager);
    println!("last price: {}", token_info.last_price);

    let token_amount = U256::from(1_000_000_000_000_000_000u128);
    let buy_quote = sdk.quote_buy(token, token_amount, U256::ZERO).await?;
    println!("estimated buy cost: {}", buy_quote.estimated_cost);

    let sell_quote = sdk.quote_sell(token, token_amount).await?;
    println!("estimated sell funds: {}", sell_quote.funds);

    Ok(())
}
