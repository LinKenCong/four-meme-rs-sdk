use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionReceipt;
use alloy::sol_types::SolEvent;
use base64::Engine;
use serde_json::json;

use crate::client::FourMemeSdk;
use crate::contracts::Eip8004Nft;
use crate::error::{Result, SdkError};
use crate::receipt::confirm_receipt;
use crate::types::{AgentMetadata, AgentRegistration};
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
        let metadata = AgentMetadata::new(name, image_url, description)?;
        let agent_uri = build_agent_uri(&metadata);
        let signer = signer_from_private_key(private_key)?;
        let provider = self.signer_provider(signer)?;
        let nft = Eip8004Nft::new(self.config.addresses.eip8004_nft, provider);
        let receipt = nft
            .register(agent_uri.clone())
            .send()
            .await
            .map_err(contract_error)?
            .get_receipt()
            .await
            .map_err(contract_error)?;
        let confirmed = confirm_receipt(receipt.clone())?;
        let agent_id = registered_agent_id(&receipt, self.config.addresses.eip8004_nft)?;
        Ok(AgentRegistration {
            tx_hash: confirmed.tx_hash,
            agent_id,
            agent_uri,
        })
    }
}

pub fn build_agent_uri(metadata: &AgentMetadata) -> String {
    let description = if metadata.description.is_empty() {
        "I'm four.meme trading agent"
    } else {
        &metadata.description
    };
    let payload = json!({
        "type": REGISTRATION_TYPE,
        "name": &metadata.name,
        "description": description,
        "image": &metadata.image_url,
        "active": true,
        "supportedTrust": [""]
    });
    let encoded = base64::engine::general_purpose::STANDARD.encode(payload.to_string());
    format!("data:application/json;base64,{encoded}")
}

fn registered_agent_id(receipt: &TransactionReceipt, nft_address: Address) -> Result<U256> {
    receipt
        .inner
        .logs()
        .iter()
        .filter(|log| log.address() == nft_address)
        .find_map(|log| {
            Eip8004Nft::Registered::decode_log(&log.inner)
                .ok()
                .map(|registered| registered.data.agentId)
        })
        .ok_or(SdkError::MissingRegisteredEvent)
}

fn contract_error(error: impl std::fmt::Display) -> SdkError {
    SdkError::Contract(error.to_string())
}
