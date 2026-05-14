mod support;

use alloy::primitives::{Bytes, U256, address};
use alloy::sol_types::SolCall;
use four_meme_sdk::contracts::TokenManager2;
use four_meme_sdk::trade::{
    encode_approval_calldata, encode_buy_token_amap_calldata, encode_buy_token_calldata,
    encode_sell_token_calldata,
};
use four_meme_sdk::types::{
    ApiCode, ApiEnvelope, BuyExecutionPlan, CreateTokenApiOutput, SellExecutionPlan, TokenLabel,
    TokenTaxInfo, TradeApproval,
};
use four_meme_sdk::utils::{normalize_hex_or_base64, parse_address, parse_bnb_to_wei, parse_u256};
use four_meme_sdk::{SdkConfig, SdkError, api::encode_create_token_calldata};
use support::fixtures::token_create_log_fixture;

#[test]
fn sdk_rejects_unsupported_chain_before_network_setup() {
    let error = four_meme_sdk::FourMemeSdk::new(SdkConfig::new().with_chain_id_for_test(97))
        .expect_err("unsupported chain should fail");

    assert!(matches!(error, SdkError::UnsupportedChain(97)));
}

#[test]
fn api_envelope_accepts_string_and_numeric_success_codes() {
    let string_code = ApiEnvelope {
        code: ApiCode::String("0".to_string()),
        msg: None,
        message: None,
        data: Some(()),
    };
    let number_code = ApiEnvelope {
        code: ApiCode::Signed(0),
        msg: None,
        message: None,
        data: Some(()),
    };

    assert!(string_code.is_success());
    assert!(number_code.is_success());
    assert_eq!(number_code.code_string(), "0");
}

#[test]
fn token_label_parser_normalizes_case_and_aliases() {
    assert_eq!(TokenLabel::try_from(" ai ").unwrap(), TokenLabel::Ai);
    assert_eq!(TokenLabel::try_from("desci").unwrap(), TokenLabel::DeSci);
    assert_eq!(TokenLabel::Ai.as_api_str(), "AI");
}

#[test]
fn token_tax_validation_enforces_supported_fee_rate_and_sum() {
    let valid = TokenTaxInfo {
        fee_rate: 3,
        burn_rate: 25,
        divide_rate: 25,
        liquidity_rate: 25,
        recipient_rate: 25,
        recipient_address: Some("0x0000000000000000000000000000000000000001".to_string()),
        min_sharing: 0,
    };
    assert!(valid.validate().is_ok());

    let invalid_sum = TokenTaxInfo {
        burn_rate: 10,
        ..valid.clone()
    };
    assert!(matches!(
        invalid_sum.validate(),
        Err(SdkError::Validation {
            field: "token_tax_info",
            ..
        })
    ));

    let invalid_fee = TokenTaxInfo {
        fee_rate: 2,
        ..valid
    };
    assert!(matches!(
        invalid_fee.validate(),
        Err(SdkError::Validation {
            field: "fee_rate",
            ..
        })
    ));
}

#[test]
fn parsing_helpers_report_boundary_errors() {
    assert!(parse_address("not an address").is_err());
    assert!(parse_u256("not an amount").is_err());
    assert_eq!(parse_bnb_to_wei("0").unwrap().to_string(), "0");
    assert_eq!(
        parse_bnb_to_wei("1").unwrap().to_string(),
        "1000000000000000000"
    );
}

#[test]
fn payload_normalizer_accepts_hex_and_base64_fixtures() {
    let hex_bytes = normalize_hex_or_base64("0x0102").expect("hex should decode");
    let base64_bytes = normalize_hex_or_base64("AQI=").expect("base64 should decode");

    assert_eq!(hex_bytes, base64_bytes);
}

#[test]
fn trade_plans_expose_canonical_calldata() {
    let token = address!("0000000000000000000000000000000000000001");
    let spender = address!("0000000000000000000000000000000000000002");
    let amount = U256::from(123_u64);
    let max_funds = U256::from(456_u64);
    let funds = U256::from(789_u64);
    let min_amount = U256::from(10_u64);
    let min_funds = U256::from(11_u64);

    let approval = TradeApproval {
        token,
        spender,
        amount,
        calldata: encode_approval_calldata(spender, amount),
    };
    assert_eq!(approval.calldata, approval.expected_calldata());

    let fixed_amount = BuyExecutionPlan::FixedAmount {
        token,
        value: U256::ZERO,
        amount,
        max_funds,
        calldata: encode_buy_token_calldata(token, amount, max_funds),
    };
    assert_eq!(
        fixed_amount.expected_calldata(),
        encode_buy_token_calldata(token, amount, max_funds)
    );

    let fixed_funds = BuyExecutionPlan::FixedFunds {
        token,
        value: U256::ZERO,
        funds,
        min_amount,
        calldata: encode_buy_token_amap_calldata(token, funds, min_amount),
    };
    assert_eq!(
        fixed_funds.expected_calldata(),
        encode_buy_token_amap_calldata(token, funds, min_amount)
    );

    let sell = SellExecutionPlan {
        token,
        value: U256::ZERO,
        amount,
        min_funds: Some(min_funds),
        calldata: encode_sell_token_calldata(token, amount, Some(min_funds)),
    };
    assert_eq!(
        sell.calldata,
        encode_sell_token_calldata(token, amount, Some(min_funds))
    );
    assert_eq!(sell.calldata, sell.expected_calldata());
}

#[test]
fn create_token_output_exposes_create_token_calldata() {
    let expected = TokenManager2::createTokenCall {
        args: Bytes::from(vec![1_u8, 2]),
        signature: Bytes::from(vec![3_u8, 4]),
    }
    .abi_encode();
    let output = CreateTokenApiOutput {
        create_arg: "0x0102".to_string(),
        signature: "0x0304".to_string(),
        creation_fee_wei: "0".to_string(),
        calldata: "0x".to_string(),
    };

    assert_eq!(
        output.expected_calldata().unwrap(),
        Bytes::from(expected.clone())
    );
    assert_eq!(
        encode_create_token_calldata(&output.create_arg, &output.signature).unwrap(),
        Bytes::from(expected)
    );
}

#[test]
fn missing_calldata_fields_deserialize_to_empty_defaults() {
    let approval = TradeApproval {
        token: address!("0000000000000000000000000000000000000001"),
        spender: address!("0000000000000000000000000000000000000002"),
        amount: U256::from(1_u64),
        calldata: Bytes::from(vec![1_u8]),
    };
    let mut approval_json = serde_json::to_value(&approval).expect("approval serializes");
    approval_json
        .as_object_mut()
        .expect("approval object")
        .remove("calldata");
    let decoded: TradeApproval =
        serde_json::from_value(approval_json).expect("missing calldata defaults");

    let create_json = serde_json::json!({
        "createArg": "0x01",
        "signature": "0x02",
        "creationFeeWei": "0"
    });
    let decoded_create: CreateTokenApiOutput =
        serde_json::from_value(create_json).expect("missing create calldata defaults");

    assert!(decoded.calldata.is_empty());
    assert!(decoded_create.calldata.is_empty());
}

#[test]
fn log_fixture_matches_public_event_shape() {
    let event = token_create_log_fixture();

    assert_eq!(event["eventName"], "TokenCreate");
    assert_eq!(event["blockNumber"], 1);
    assert!(event["transactionHash"].as_str().unwrap().starts_with("0x"));
}

trait TestConfigExt {
    fn with_chain_id_for_test(self, chain_id: u64) -> Self;
}

impl TestConfigExt for SdkConfig {
    fn with_chain_id_for_test(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }
}
