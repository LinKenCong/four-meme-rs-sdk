#![allow(dead_code)]

use alloy::primitives::Address;
use four_meme_sdk::utils::parse_address;
use four_meme_sdk::{FourMemeSdk, Result, SdkConfig};

pub const EXAMPLE_PRIVATE_KEY_ENV: &str = "FOUR_MEME_EXAMPLE_PRIVATE_KEY";
pub const EXAMPLE_TOKEN_ADDRESS_ENV: &str = "FOUR_MEME_EXAMPLE_TOKEN_ADDRESS";
pub const EXAMPLE_OWNER_ADDRESS_ENV: &str = "FOUR_MEME_EXAMPLE_OWNER_ADDRESS";
pub const FORK_RPC_URL_ENV: &str = "FOUR_MEME_FORK_RPC_URL";
pub const ENABLE_PREPARE_CREATE_ENV: &str = "FOUR_MEME_EXAMPLE_ENABLE_PREPARE_CREATE";

pub fn build_read_only_sdk() -> Result<FourMemeSdk> {
    FourMemeSdk::from_env()
}

pub fn build_local_fork_sdk() -> Result<FourMemeSdk> {
    let rpc_url =
        std::env::var(FORK_RPC_URL_ENV).unwrap_or_else(|_| "http://127.0.0.1:8545".to_string());
    FourMemeSdk::new(SdkConfig::local_fork().with_rpc_url(rpc_url))
}

pub fn example_private_key() -> Option<String> {
    non_empty_env(EXAMPLE_PRIVATE_KEY_ENV)
}

pub fn example_token_address() -> Result<Option<Address>> {
    optional_address_env(EXAMPLE_TOKEN_ADDRESS_ENV)
}

pub fn example_owner_address() -> Result<Address> {
    Ok(optional_address_env(EXAMPLE_OWNER_ADDRESS_ENV)?.unwrap_or(Address::ZERO))
}

pub fn skip_missing_address(example_name: &str, env_name: &str) {
    eprintln!("Skipping {example_name}: set {env_name} to a token address to run this example.");
}

pub fn skip_write_example(example_name: &str) {
    eprintln!(
        "Skipping {example_name}: set {EXAMPLE_PRIVATE_KEY_ENV} and run against a local fork only."
    );
}

pub fn skip_prepare_create_example() {
    eprintln!(
        "Skipping prepare_create: set {ENABLE_PREPARE_CREATE_ENV}=1 and {EXAMPLE_PRIVATE_KEY_ENV} to call the Four.meme preparation API. This example does not broadcast a chain transaction."
    );
}

pub fn opt_in_enabled(env_name: &str) -> bool {
    non_empty_env(env_name)
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn optional_address_env(env_name: &str) -> Result<Option<Address>> {
    non_empty_env(env_name).map(parse_address).transpose()
}

fn non_empty_env(env_name: &str) -> Option<String> {
    std::env::var(env_name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
