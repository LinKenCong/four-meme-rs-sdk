#![allow(dead_code)]

use serde_json::{Value, json};

pub const TEST_TOKEN_ADDRESS: &str = "0x0000000000000000000000000000000000000001";
pub const TEST_TX_HASH: &str = "0x0000000000000000000000000000000000000000000000000000000000000001";

pub fn public_config_envelope() -> Value {
    json!({
        "code": "0",
        "msg": "success",
        "data": [{
            "symbol": "BNB",
            "symbolAddress": TEST_TOKEN_ADDRESS,
            "totalAmount": "1000000000",
            "totalBAmount": "24",
            "saleRate": "0.8",
            "status": "PUBLISH"
        }]
    })
}

pub fn token_search_response() -> Value {
    json!({
        "code": "0",
        "data": {
            "total": 1,
            "list": [{
                "symbol": "BNB",
                "tokenAddress": TEST_TOKEN_ADDRESS
            }]
        }
    })
}

pub fn token_detail_response() -> Value {
    json!({
        "code": "0",
        "data": {
            "address": TEST_TOKEN_ADDRESS,
            "symbol": "BNB"
        }
    })
}

pub fn token_create_log_fixture() -> Value {
    json!({
        "eventName": "TokenCreate",
        "blockNumber": 1,
        "transactionHash": TEST_TX_HASH,
        "args": {
            "creator": TEST_TOKEN_ADDRESS,
            "symbol": "BNB"
        }
    })
}

pub fn api_error_envelope() -> Value {
    json!({
        "code": "40001",
        "msg": "validation failed",
        "data": []
    })
}
