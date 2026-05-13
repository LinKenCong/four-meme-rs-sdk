mod support;

use alloy::primitives::address;
use support::anvil::{ANVIL_FORK_URL_ENV, AnvilFork, should_run_anvil_e2e};

#[tokio::test]
#[ignore = "requires Anvil and a FOUR_MEME_ANVIL_FORK_URL endpoint; never runs in default CI"]
async fn anvil_fork_reads_contract_state_without_mainnet_writes() {
    if !should_run_anvil_e2e() {
        eprintln!("set {ANVIL_FORK_URL_ENV} to a BSC RPC endpoint to run this ignored e2e test");
        return;
    }

    let fork = AnvilFork::start_from_env().expect("anvil should start from the fork URL");
    let owner = address!("0000000000000000000000000000000000000000");
    let balance = fork
        .sdk()
        .eip8004_balance(owner)
        .await
        .expect("forked read-only contract call should succeed");

    assert_eq!(balance.to_string(), "0");
}
