use alloy::primitives::{Address, U256};
use alloy::signers::Signer;
use reqwest::multipart::{Form, Part};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::client::FourMemeSdk;
use crate::contracts::TokenManager2;
use crate::error::{RedactedContext, Result, SdkError};
use crate::types::{
    ApiEnvelope, CreateTokenApiOutput, CreateTokenImage, CreateTokenRequest, RaisedToken,
    RankingRequest, TokenSearchRequest,
};
use crate::utils::{normalize_hex_or_base64, parse_bnb_to_wei};
use crate::wallet::signer_from_private_key;

impl FourMemeSdk {
    pub async fn public_config(&self) -> Result<Vec<RaisedToken>> {
        self.get_api_data("/public/config").await
    }

    pub async fn token_detail(&self, address: Address) -> Result<Value> {
        let url = format!("/private/token/get/v2?address={address}");
        self.get_raw(&url).await
    }

    pub async fn token_search(&self, request: &TokenSearchRequest) -> Result<Value> {
        self.post_raw("/public/token/search", request).await
    }

    pub async fn token_rankings(&self, request: &RankingRequest) -> Result<Value> {
        self.post_raw("/public/token/ranking", request).await
    }

    pub async fn prepare_create_token(
        &self,
        private_key: impl AsRef<str>,
        request: CreateTokenRequest,
    ) -> Result<CreateTokenApiOutput> {
        if request.name.trim().is_empty() {
            return Err(SdkError::validation("name", "missing required field"));
        }
        if request.short_name.trim().is_empty() {
            return Err(SdkError::validation("short_name", "missing required field"));
        }
        if request.desc.trim().is_empty() {
            return Err(SdkError::validation("desc", "missing required field"));
        }
        if let Some(tax) = &request.token_tax_info {
            tax.validate()?;
        }

        let signer = signer_from_private_key(private_key)?;
        let address = signer.address();
        let access_token = self.login(address, &signer).await?;
        let img_url = match &request.image {
            CreateTokenImage::File { file_name, bytes } => {
                self.upload_token_image(&access_token, file_name, bytes.clone())
                    .await?
            }
            CreateTokenImage::Url(url) => url.clone(),
        };
        let raised_token = self.preferred_raised_token().await?;
        let body = self.build_create_token_body(&request, img_url, raised_token)?;
        let response: CreateTokenApiData = self
            .post_api_data_with_access("/private/token/create", &access_token, &body)
            .await?;
        let create_arg = hex_string(normalize_hex_or_base64(response.create_arg)?);
        let signature = hex_string(normalize_hex_or_base64(response.signature)?);
        let creation_fee_wei = self.estimate_creation_fee_wei(&body).await?;
        Ok(CreateTokenApiOutput {
            create_arg,
            signature,
            creation_fee_wei: creation_fee_wei.to_string(),
        })
    }

    async fn login(&self, address: Address, signer: &(impl Signer + Sync)) -> Result<String> {
        let nonce_body = json!({
            "accountAddress": address.to_string(),
            "verifyType": "LOGIN",
            "networkCode": "BSC"
        });
        let nonce: Value = self
            .post_api_data("/private/user/nonce/generate", &nonce_body)
            .await?;
        let nonce_text = match nonce {
            Value::String(value) => value,
            other => other.to_string(),
        };
        let message = format!("You are sign in Meme {nonce_text}");
        let signature = signer
            .sign_message(message.as_bytes())
            .await
            .map_err(|error| SdkError::signing("login message", error))?;
        let login_body = json!({
            "region": "WEB",
            "langType": "EN",
            "loginIp": "",
            "inviteCode": "",
            "verifyInfo": {
                "address": address.to_string(),
                "networkCode": "BSC",
                "signature": signature.to_string(),
                "verifyType": "LOGIN"
            },
            "walletName": "MetaMask"
        });
        self.post_api_data("/private/user/login/dex", &login_body)
            .await
    }

    async fn upload_token_image(
        &self,
        access_token: &str,
        file_name: &str,
        bytes: Vec<u8>,
    ) -> Result<String> {
        let part = Part::bytes(bytes).file_name(file_name.to_string());
        let form = Form::new().part("file", part);
        let response = self
            .http
            .post(self.api_url("/private/token/upload"))
            .header("meme-web-access", access_token)
            .multipart(form)
            .send()
            .await
            .map_err(|error| SdkError::http("upload token image", error))?;
        self.decode_envelope("/private/token/upload", response)
            .await
    }

    async fn preferred_raised_token(&self) -> Result<RaisedToken> {
        let tokens = self.public_config().await?;
        let published: Vec<_> = tokens
            .iter()
            .filter(|token| token.status.as_deref() == Some("PUBLISH"))
            .cloned()
            .collect();
        let candidates = if published.is_empty() {
            tokens
        } else {
            published
        };
        candidates
            .iter()
            .find(|token| token.symbol == "BNB")
            .cloned()
            .or_else(|| candidates.into_iter().next())
            .ok_or_else(|| SdkError::config("raised_token", "no raised token config is available"))
    }

    fn build_create_token_body(
        &self,
        request: &CreateTokenRequest,
        img_url: String,
        raised_token: RaisedToken,
    ) -> Result<Value> {
        let total_supply = number_field(&raised_token.total_amount).unwrap_or(1_000_000_000.0);
        let raised_amount = number_field(&raised_token.total_b_amount).unwrap_or(24.0);
        let sale_rate = number_field(&raised_token.sale_rate).unwrap_or(0.8);
        let mut body = json!({
            "name": request.name,
            "shortName": request.short_name,
            "desc": request.desc,
            "totalSupply": total_supply,
            "raisedAmount": raised_amount,
            "saleRate": sale_rate,
            "reserveRate": 0,
            "imgUrl": img_url,
            "raisedToken": raised_token,
            "launchTime": epoch_millis(),
            "funGroup": false,
            "label": request.label,
            "lpTradingFee": 0.0025,
            "preSale": request.pre_sale,
            "clickFun": false,
            "symbol": body_symbol(&raised_token),
            "dexType": "PANCAKE_SWAP",
            "rushMode": false,
            "onlyMPC": false,
            "feePlan": request.fee_plan
        });
        if let Some(url) = &request.web_url {
            body["webUrl"] = json!(url);
        }
        if let Some(url) = &request.twitter_url {
            body["twitterUrl"] = json!(url);
        }
        if let Some(url) = &request.telegram_url {
            body["telegramUrl"] = json!(url);
        }
        if let Some(tax) = &request.token_tax_info {
            body["tokenTaxInfo"] = serde_json::to_value(tax)?;
        }
        Ok(body)
    }

    async fn estimate_creation_fee_wei(&self, body: &Value) -> Result<U256> {
        let manager =
            TokenManager2::new(self.config.addresses.token_manager2, self.provider.clone());
        let launch_fee = manager
            ._launchFee()
            .call()
            .await
            .map_err(|error| SdkError::rpc_provider("contract call", error))?;
        let presale_wei = create_presale_wei(body)?;
        if presale_wei == U256::ZERO || body["symbol"].as_str() != Some("BNB") {
            return Ok(launch_fee);
        }
        let fee_rate = manager
            ._tradingFeeRate()
            .call()
            .await
            .map_err(|error| SdkError::rpc_provider("contract call", error))?;
        calculate_creation_fee_wei(launch_fee, presale_wei, fee_rate)
    }

    async fn get_api_data<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self
            .http
            .get(self.api_url(path))
            .send()
            .await
            .map_err(|error| SdkError::http("GET Four.meme API", error))?;
        self.decode_envelope(path, response).await
    }

    async fn post_api_data<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let response = self
            .http
            .post(self.api_url(path))
            .json(body)
            .send()
            .await
            .map_err(|error| SdkError::http("POST Four.meme API", error))?;
        self.decode_envelope(path, response).await
    }

    async fn post_api_data_with_access<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        access_token: &str,
        body: &B,
    ) -> Result<T> {
        let response = self
            .http
            .post(self.api_url(path))
            .header("meme-web-access", access_token)
            .json(body)
            .send()
            .await
            .map_err(|error| SdkError::http("POST authenticated Four.meme API", error))?;
        self.decode_envelope(path, response).await
    }

    async fn get_raw(&self, path: &str) -> Result<Value> {
        let response = self
            .http
            .get(self.api_url(path))
            .send()
            .await
            .map_err(|error| SdkError::http("GET raw Four.meme API", error))?;
        decode_raw_response("GET raw Four.meme API", response).await
    }

    async fn post_raw<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<Value> {
        let response = self
            .http
            .post(self.api_url(path))
            .json(body)
            .send()
            .await
            .map_err(|error| SdkError::http("POST raw Four.meme API", error))?;
        decode_raw_response("POST raw Four.meme API", response).await
    }

    async fn decode_envelope<T: DeserializeOwned>(
        &self,
        path: &str,
        response: reqwest::Response,
    ) -> Result<T> {
        let response = response
            .error_for_status()
            .map_err(|error| SdkError::http("Four.meme API status", error))?;
        let text = response
            .text()
            .await
            .map_err(|error| SdkError::http("Four.meme API body", error))?;
        let envelope: ApiEnvelope<T> = serde_json::from_str(&text)
            .map_err(|error| SdkError::serialization("Four.meme API envelope", error))?;
        if !envelope.is_success() {
            return Err(SdkError::rest_business(
                envelope.code_string(),
                envelope.message_text(),
                RedactedContext::new([("path", path), ("response_body", text.as_str())]),
            ));
        }
        Ok(envelope.data)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTokenApiData {
    create_arg: String,
    signature: String,
}

fn hex_string(bytes: alloy::primitives::Bytes) -> String {
    format!("0x{}", hex::encode(bytes))
}

fn create_presale_wei(body: &Value) -> Result<U256> {
    match body["preSale"].as_str().map(str::trim) {
        Some("") | None => Ok(U256::ZERO),
        Some(value) => parse_bnb_to_wei(value),
    }
}

fn calculate_creation_fee_wei(launch_fee: U256, presale_wei: U256, fee_rate: U256) -> Result<U256> {
    let presale_fee = presale_wei
        .checked_mul(fee_rate)
        .and_then(|value| value.checked_div(U256::from(10_000u16)))
        .ok_or_else(|| SdkError::InvalidAmount(presale_wei.to_string()))?;
    launch_fee
        .checked_add(presale_wei)
        .and_then(|value| value.checked_add(presale_fee))
        .ok_or_else(|| SdkError::InvalidAmount(presale_wei.to_string()))
}

fn number_field(value: &Option<Value>) -> Option<f64> {
    match value.as_ref()? {
        Value::Number(number) => number.as_f64(),
        Value::String(value) => value.parse().ok(),
        _ => None,
    }
}

fn body_symbol(token: &RaisedToken) -> &str {
    token.symbol.as_str()
}

async fn decode_raw_response(
    operation: &'static str,
    response: reqwest::Response,
) -> Result<Value> {
    let response = response
        .error_for_status()
        .map_err(|error| SdkError::http(operation, error))?;
    response
        .json::<Value>()
        .await
        .map_err(|error| SdkError::http(operation, error))
}

fn epoch_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_presale_wei_parses_exact_bnb_amounts() {
        let body = json!({ "preSale": "0.000000000000000001" });
        assert_eq!(create_presale_wei(&body).unwrap(), U256::from(1u8));
    }

    #[test]
    fn create_presale_wei_treats_missing_or_blank_presale_as_zero() {
        assert_eq!(create_presale_wei(&json!({})).unwrap(), U256::ZERO);
        assert_eq!(
            create_presale_wei(&json!({ "preSale": "  " })).unwrap(),
            U256::ZERO
        );
    }

    #[test]
    fn create_presale_wei_rejects_fraction_beyond_wei() {
        let body = json!({ "preSale": "0.0000000000000000001" });
        assert!(create_presale_wei(&body).is_err());
    }

    #[test]
    fn calculate_creation_fee_wei_adds_presale_and_fee_without_float_rounding() {
        let launch_fee = U256::from(1_000u64);
        let presale_wei = parse_bnb_to_wei("1.234567890123456789").unwrap();
        let fee_rate = U256::from(250u16);

        assert_eq!(
            calculate_creation_fee_wei(launch_fee, presale_wei, fee_rate).unwrap(),
            U256::from(1_265_432_087_376_544_208u128)
        );
    }
}
