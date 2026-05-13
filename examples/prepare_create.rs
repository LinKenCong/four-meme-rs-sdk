mod common;

use four_meme_sdk::types::{CreateTokenImage, CreateTokenRequest, TokenLabel};

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    if !common::opt_in_enabled(common::ENABLE_PREPARE_CREATE_ENV) {
        common::skip_prepare_create_example();
        return Ok(());
    }

    let Some(private_key) = common::example_private_key() else {
        common::skip_prepare_create_example();
        return Ok(());
    };

    let sdk = common::build_read_only_sdk()?;
    let request = CreateTokenRequest {
        name: "Example Token".to_string(),
        short_name: "EXAMPLE".to_string(),
        desc: "Compile-checked example token. Do not submit on mainnet.".to_string(),
        label: TokenLabel::Meme,
        image: CreateTokenImage::Url("https://example.com/token.png".to_string()),
        web_url: Some("https://example.com".to_string()),
        twitter_url: None,
        telegram_url: None,
        pre_sale: "0".to_string(),
        fee_plan: false,
        token_tax_info: None,
    };

    let prepared = sdk.prepare_create_token(private_key, request).await?;
    println!("creation fee wei: {}", prepared.creation_fee_wei);
    println!("create arg bytes: {}", prepared.create_arg.len());
    println!("signature bytes: {}", prepared.signature.len());

    Ok(())
}
