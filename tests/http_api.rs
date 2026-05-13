mod support;

use alloy::primitives::Address;
use four_meme_sdk::{Result, SdkError};
use serde_json::json;
use support::fixtures::{
    TEST_TOKEN_ADDRESS, api_error_envelope, public_config_envelope, token_detail_response,
    token_search_response,
};
use support::http::MockFourMemeApi;

#[tokio::test]
async fn public_config_reads_mocked_envelope_without_network() -> Result<()> {
    let api = MockFourMemeApi::start();
    let mock = api.get_json("/public/config", public_config_envelope());
    let config = api.sdk().public_config().await?;

    mock.assert();
    assert_eq!(config.len(), 1);
    assert_eq!(config.raised_tokens()[0].symbol, "BNB");
    assert_eq!(config.raised_tokens()[0].status.as_deref(), Some("PUBLISH"));
    Ok(())
}

#[tokio::test]
async fn token_search_posts_request_to_mock_server() -> Result<()> {
    let api = MockFourMemeApi::start();
    let mock = api.post_json("/public/token/search", token_search_response());
    let response = api
        .sdk()
        .token_search(&four_meme_sdk::types::TokenSearchRequest::default())
        .await?;

    mock.assert();
    assert_eq!(response.total, Some(1));
    assert_eq!(response.list[0].symbol.as_deref(), Some("BNB"));
    Ok(())
}

#[tokio::test]
async fn token_detail_uses_private_detail_path() -> Result<()> {
    let api = MockFourMemeApi::start();
    let token = TEST_TOKEN_ADDRESS
        .parse::<Address>()
        .expect("valid address");
    let mock = api.get_json_with_query(
        "/private/token/get/v2",
        ("address", token.to_string()),
        token_detail_response(),
    );
    let response = api.sdk().token_detail(token).await?;

    mock.assert();
    assert_eq!(response.token_address.as_deref(), Some(TEST_TOKEN_ADDRESS));
    Ok(())
}

#[tokio::test]
async fn api_errors_preserve_code_and_body() {
    let api = MockFourMemeApi::start();
    let mock = api.get_json("/public/config", api_error_envelope());
    let error = api
        .sdk()
        .public_config()
        .await
        .expect_err("mocked API error should fail");

    mock.assert();
    match error {
        SdkError::RestBusiness { code, context, .. } => {
            assert_eq!(code, "40001");
            assert!(context.to_string().contains("validation failed"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[tokio::test]
async fn token_rankings_posts_request_to_mock_server() -> Result<()> {
    let api = MockFourMemeApi::start();
    let mock = api.post_json(
        "/public/token/ranking",
        json!({
            "code": "0",
            "data": {
                "items": [{ "symbol": "BNB" }]
            }
        }),
    );
    let response = api
        .sdk()
        .token_rankings(&four_meme_sdk::types::RankingRequest::new("marketCap"))
        .await?;

    mock.assert();
    assert_eq!(response.list[0].symbol.as_deref(), Some("BNB"));
    Ok(())
}
