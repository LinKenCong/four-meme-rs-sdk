mod support;

use four_meme_sdk::types::{ApiEnvelope, TokenLabel, TokenTaxInfo};
use four_meme_sdk::utils::{bnb_to_wei_lossy, normalize_hex_or_base64, parse_address, parse_u256};
use four_meme_sdk::{SdkConfig, SdkError};
use serde_json::json;
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
        code: json!("0"),
        msg: None,
        message: None,
        data: (),
    };
    let number_code = ApiEnvelope {
        code: json!(0),
        msg: None,
        message: None,
        data: (),
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
        recipient_address: None,
        min_sharing: 0,
    };
    assert!(valid.validate().is_ok());

    let invalid_sum = TokenTaxInfo {
        burn_rate: 10,
        ..valid.clone()
    };
    assert!(matches!(
        invalid_sum.validate(),
        Err(SdkError::InvalidTaxRateSum(85))
    ));

    let invalid_fee = TokenTaxInfo {
        fee_rate: 2,
        ..valid
    };
    assert!(matches!(
        invalid_fee.validate(),
        Err(SdkError::InvalidTaxFeeRate(2))
    ));
}

#[test]
fn parsing_helpers_report_boundary_errors() {
    assert!(parse_address("not an address").is_err());
    assert!(parse_u256("not an amount").is_err());
    assert_eq!(bnb_to_wei_lossy(0.0).to_string(), "0");
    assert_eq!(bnb_to_wei_lossy(1.0).to_string(), "1000000000000000000");
}

#[test]
fn payload_normalizer_accepts_hex_and_base64_fixtures() {
    let hex_bytes = normalize_hex_or_base64("0x0102").expect("hex should decode");
    let base64_bytes = normalize_hex_or_base64("AQI=").expect("base64 should decode");

    assert_eq!(hex_bytes, base64_bytes);
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
