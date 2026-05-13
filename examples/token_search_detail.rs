mod common;

use four_meme_sdk::types::TokenSearchRequest;

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let sdk = common::build_read_only_sdk()?;
    let search_request = TokenSearchRequest {
        keyword: Some("meme".to_string()),
        page_size: 5,
        ..TokenSearchRequest::default()
    };

    let search_response = sdk.token_search(&search_request).await?;
    println!(
        "found {:?} tokens on page {:?}",
        search_response.total, search_response.page_index
    );
    for token in &search_response.list {
        println!(
            "token={} symbol={}",
            token.token_address.as_deref().unwrap_or("unknown"),
            token.symbol.as_deref().unwrap_or("unknown")
        );
    }

    let Some(token_address) = common::example_token_address()? else {
        common::skip_missing_address("token detail", common::EXAMPLE_TOKEN_ADDRESS_ENV);
        return Ok(());
    };
    let detail_response = sdk.token_detail(token_address).await?;
    println!(
        "token detail: address={} symbol={}",
        detail_response
            .token_address
            .as_deref()
            .unwrap_or("unknown"),
        detail_response.symbol.as_deref().unwrap_or("unknown")
    );

    Ok(())
}
