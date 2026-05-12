use alloy::primitives::{B256, b256, keccak256};
use alloy::sol_types::{SolCall, SolEvent};
use four_meme_sdk::contracts::{Eip8004Nft, Erc20, TaxToken, TokenManager2, TokenManagerHelper3};

// ABI sources are the Four.meme BSC contract interfaces mirrored in src/contracts.rs.
// Snapshot values are derived from canonical Solidity signatures and intentionally
// checked against both literals and runtime Keccak output to catch signature drift.

struct FunctionSnapshot {
    signature: &'static str,
    expected_selector: [u8; 4],
    actual_selector: [u8; 4],
}

struct EventSnapshot {
    signature: &'static str,
    expected_topic: B256,
    actual_topic: B256,
}

fn selector_from_signature(signature: &str) -> [u8; 4] {
    keccak256(signature.as_bytes())[..4]
        .try_into()
        .expect("Keccak-256 output always contains a 4-byte selector")
}

fn assert_function_snapshot(snapshot: FunctionSnapshot) {
    assert_eq!(
        snapshot.actual_selector, snapshot.expected_selector,
        "selector snapshot changed for {}",
        snapshot.signature
    );
    assert_eq!(
        snapshot.actual_selector,
        selector_from_signature(snapshot.signature),
        "selector is not derived from the canonical signature for {}",
        snapshot.signature
    );
}

fn assert_event_snapshot(snapshot: EventSnapshot) {
    assert_eq!(
        snapshot.actual_topic, snapshot.expected_topic,
        "event topic snapshot changed for {}",
        snapshot.signature
    );
    assert_eq!(
        snapshot.actual_topic,
        keccak256(snapshot.signature.as_bytes()),
        "event topic is not derived from the canonical signature for {}",
        snapshot.signature
    );
}

#[test]
fn token_manager_helper3_selectors_match_snapshots() {
    let snapshots = [
        FunctionSnapshot {
            signature: "getTokenInfo(address)",
            expected_selector: [0x1f, 0x69, 0x56, 0x5f],
            actual_selector: TokenManagerHelper3::getTokenInfoCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "tryBuy(address,uint256,uint256)",
            expected_selector: [0xe2, 0x1b, 0x10, 0x3a],
            actual_selector: TokenManagerHelper3::tryBuyCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "trySell(address,uint256)",
            expected_selector: [0xc6, 0xf4, 0x3e, 0x8c],
            actual_selector: TokenManagerHelper3::trySellCall::SELECTOR,
        },
    ];

    for snapshot in snapshots {
        assert_function_snapshot(snapshot);
    }
}

#[test]
fn token_manager2_selectors_match_snapshots() {
    let snapshots = [
        FunctionSnapshot {
            signature: "_launchFee()",
            expected_selector: [0x00, 0x95, 0x23, 0xa2],
            actual_selector: TokenManager2::_launchFeeCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "_tradingFeeRate()",
            expected_selector: [0x34, 0x72, 0xae, 0xe7],
            actual_selector: TokenManager2::_tradingFeeRateCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "createToken(bytes,bytes)",
            expected_selector: [0x51, 0x9e, 0xbb, 0x10],
            actual_selector: TokenManager2::createTokenCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "buyToken(address,uint256,uint256)",
            expected_selector: [0xe6, 0x71, 0x49, 0x9b],
            actual_selector: TokenManager2::buyTokenCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "buyTokenAMAP(address,uint256,uint256)",
            expected_selector: [0x87, 0xf2, 0x76, 0x55],
            actual_selector: TokenManager2::buyTokenAMAPCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "sellToken(address,uint256)",
            expected_selector: [0xf4, 0x64, 0xe7, 0xdb],
            actual_selector: TokenManager2::sellToken_0Call::SELECTOR,
        },
        FunctionSnapshot {
            signature: "sellToken(uint256,address,uint256,uint256)",
            expected_selector: [0x0d, 0xa7, 0x49, 0x35],
            actual_selector: TokenManager2::sellToken_1Call::SELECTOR,
        },
    ];

    for snapshot in snapshots {
        assert_function_snapshot(snapshot);
    }
}

#[test]
fn tax_token_selectors_match_snapshots() {
    let snapshots = [
        FunctionSnapshot {
            signature: "feeRate()",
            expected_selector: [0x97, 0x8b, 0xbd, 0xb9],
            actual_selector: TaxToken::feeRateCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "rateFounder()",
            expected_selector: [0x6f, 0x0e, 0x50, 0x53],
            actual_selector: TaxToken::rateFounderCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "rateHolder()",
            expected_selector: [0x62, 0x34, 0xb8, 0x4f],
            actual_selector: TaxToken::rateHolderCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "rateBurn()",
            expected_selector: [0x18, 0xa4, 0xac, 0xea],
            actual_selector: TaxToken::rateBurnCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "rateLiquidity()",
            expected_selector: [0xed, 0xa5, 0x28, 0xd4],
            actual_selector: TaxToken::rateLiquidityCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "minDispatch()",
            expected_selector: [0x11, 0x03, 0x95, 0xbd],
            actual_selector: TaxToken::minDispatchCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "minShare()",
            expected_selector: [0x8b, 0xb2, 0x8d, 0xe2],
            actual_selector: TaxToken::minShareCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "quote()",
            expected_selector: [0x99, 0x9b, 0x93, 0xaf],
            actual_selector: TaxToken::quoteCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "founder()",
            expected_selector: [0x4d, 0x85, 0x3e, 0xe5],
            actual_selector: TaxToken::founderCall::SELECTOR,
        },
    ];

    for snapshot in snapshots {
        assert_function_snapshot(snapshot);
    }
}

#[test]
fn erc20_selectors_match_snapshots() {
    let snapshots = [
        FunctionSnapshot {
            signature: "approve(address,uint256)",
            expected_selector: [0x09, 0x5e, 0xa7, 0xb3],
            actual_selector: Erc20::approveCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "transfer(address,uint256)",
            expected_selector: [0xa9, 0x05, 0x9c, 0xbb],
            actual_selector: Erc20::transferCall::SELECTOR,
        },
    ];

    for snapshot in snapshots {
        assert_function_snapshot(snapshot);
    }
}

#[test]
fn eip8004_nft_selectors_match_snapshots() {
    let snapshots = [
        FunctionSnapshot {
            signature: "balanceOf(address)",
            expected_selector: [0x70, 0xa0, 0x82, 0x31],
            actual_selector: Eip8004Nft::balanceOfCall::SELECTOR,
        },
        FunctionSnapshot {
            signature: "register(string)",
            expected_selector: [0xf2, 0xc2, 0x98, 0xbe],
            actual_selector: Eip8004Nft::registerCall::SELECTOR,
        },
    ];

    for snapshot in snapshots {
        assert_function_snapshot(snapshot);
    }
}

#[test]
fn token_manager2_event_topics_match_snapshots() {
    let snapshots = [
        EventSnapshot {
            signature: "TokenCreate(address,address,uint256,string,string,uint256,uint256,uint256)",
            expected_topic: b256!(
                "0x396d5e902b675b032348d3d2e9517ee8f0c4a926603fbc075d3d282ff00cad20"
            ),
            actual_topic: TokenManager2::TokenCreate::SIGNATURE_HASH,
        },
        EventSnapshot {
            signature: "TokenPurchase(address,address,uint256,uint256,uint256,uint256,uint256,uint256)",
            expected_topic: b256!(
                "0x7db52723a3b2cdd6164364b3b766e65e540d7be48ffa89582956d8eaebe62942"
            ),
            actual_topic: TokenManager2::TokenPurchase::SIGNATURE_HASH,
        },
        EventSnapshot {
            signature: "TokenSale(address,address,uint256,uint256,uint256,uint256,uint256,uint256)",
            expected_topic: b256!(
                "0x0a5575b3648bae2210cee56bf33254cc1ddfbc7bf637c0af2ac18b14fb1bae19"
            ),
            actual_topic: TokenManager2::TokenSale::SIGNATURE_HASH,
        },
        EventSnapshot {
            signature: "LiquidityAdded(address,uint256,address,uint256)",
            expected_topic: b256!(
                "0xc18aa71171b358b706fe3dd345299685ba21a5316c66ffa9e319268b033c44b0"
            ),
            actual_topic: TokenManager2::LiquidityAdded::SIGNATURE_HASH,
        },
    ];

    for snapshot in snapshots {
        assert_event_snapshot(snapshot);
    }
}

#[test]
fn eip8004_nft_event_topics_match_snapshots() {
    let snapshots = [EventSnapshot {
        signature: "Registered(uint256,string,address)",
        expected_topic: b256!("0xca52e62c367d81bb2e328eb795f7c7ba24afb478408a26c0e201d155c449bc4a"),
        actual_topic: Eip8004Nft::Registered::SIGNATURE_HASH,
    }];

    for snapshot in snapshots {
        assert_event_snapshot(snapshot);
    }
}
