#![allow(dead_code)]

use std::env;

use alloy::node_bindings::{Anvil, AnvilInstance};
use four_meme_sdk::{FourMemeSdk, SdkConfig};

pub const ANVIL_FORK_URL_ENV: &str = "FOUR_MEME_ANVIL_FORK_URL";

pub struct AnvilFork {
    _instance: AnvilInstance,
    sdk: FourMemeSdk,
}

impl AnvilFork {
    pub fn start_from_env() -> Option<Self> {
        let fork_url = env::var(ANVIL_FORK_URL_ENV).ok()?;
        let instance = Anvil::new().fork(fork_url).try_spawn().ok()?;
        let sdk = FourMemeSdk::new(SdkConfig::new().with_rpc_url(instance.endpoint()))
            .expect("anvil endpoint should produce a valid SDK");
        Some(Self {
            _instance: instance,
            sdk,
        })
    }

    pub fn sdk(&self) -> &FourMemeSdk {
        &self.sdk
    }
}

pub fn should_run_anvil_e2e() -> bool {
    env::var(ANVIL_FORK_URL_ENV)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}
