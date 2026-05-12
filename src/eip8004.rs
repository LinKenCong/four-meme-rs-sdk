use alloy::primitives::{Address, B256, U256};
use base64::Engine;
use serde_json::json;

use crate::client::FourMemeSdk;
use crate::contracts::Eip8004Nft;
use crate::error::{Result, SdkError};
use crate::types::AgentRegistration;
use crate::wallet::signer_from_private_key;

pub const REGISTRATION_TYPE: &str = "https://eips.ethereum.org/EIPS/eip-8004#registration-v1";

impl FourMemeSdk {
    pub async fn eip8004_balance(&self, owner: Address) -> Result<U256> {
        let nft = Eip8004Nft::new(self.config.addresses.eip8004_nft, self.provider.clone());
        nft.balanceOf(owner)
            .call()
            .await
            .map_err(|error| SdkError::Contract(error.to_string()))
    }

    pub async fn register_agent(
        &self,
        private_key: impl AsRef<str>,
        name: impl AsRef<str>,
        image_url: impl AsRef<str>,
        description: impl AsRef<str>,
    ) -> Result<AgentRegistration> {
        let name = name.as_ref().trim();
        if name.is_empty() {
            return Err(SdkError::MissingField("name"));
        }
        let agent_uri =
            build_agent_uri(name, image_url.as_ref().trim(), description.as_ref().trim());
        let signer = signer_from_private_key(private_key)?;
        let provider = self.signer_provider(signer)?;
        let nft = Eip8004Nft::new(self.config.addresses.eip8004_nft, provider);
        let receipt = nft
            .register(agent_uri.clone())
            .send()
            .await
            .map_err(|error| SdkError::Contract(error.to_string()))?
            .get_receipt()
            .await
            .map_err(|error| SdkError::Contract(error.to_string()))?;
        Ok(AgentRegistration {
            tx_hash: receipt.transaction_hash,
            agent_id: None,
            agent_uri,
        })
    }
}

pub fn build_agent_uri(name: &str, image_url: &str, description: &str) -> String {
    let payload = json!({
        "type": REGISTRATION_TYPE,
        "name": name,
        "description": if description.is_empty() { "I'm four.meme trading agent" } else { description },
        "image": image_url,
        "active": true,
        "supportedTrust": [""]
    });
    let encoded = base64::engine::general_purpose::STANDARD.encode(payload.to_string());
    format!("data:application/json;base64,{encoded}")
}

#[allow(dead_code)]
fn tx_hash(hash: B256) -> B256 {
    hash
}
