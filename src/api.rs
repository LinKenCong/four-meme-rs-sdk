use alloy::primitives::Address;
use alloy::signers::Signer;
use reqwest::multipart::{Form, Part};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::client::FourMemeSdk;
use crate::contracts::TokenManager2;
use crate::error::{Result, SdkError};
use crate::types::{
    ApiEnvelope, CreateTokenApiOutput, CreateTokenImage, CreateTokenRequest, RaisedToken,
    RankingRequest, TokenSearchRequest,
};
use crate::utils::{bnb_to_wei_lossy, normalize_hex_or_base64};
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
            return Err(SdkError::MissingField("name"));
        }
        if request.short_name.trim().is_empty() {
            return Err(SdkError::MissingField("short_name"));
        }
        if request.desc.trim().is_empty() {
            return Err(SdkError::MissingField("desc"));
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
            .map_err(|error| SdkError::Transaction(error.to_string()))?;
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
            .await?;
        self.decode_envelope(response).await
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
            .ok_or(SdkError::MissingRaisedToken)
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

    async fn estimate_creation_fee_wei(&self, body: &Value) -> Result<alloy::primitives::U256> {
        let manager =
            TokenManager2::new(self.config.addresses.token_manager2, self.provider.clone());
        let launch_fee = manager
            ._launchFee()
            .call()
            .await
            .map_err(|error| SdkError::Contract(error.to_string()))?;
        let pre_sale = body["preSale"]
            .as_str()
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.0);
        let symbol_is_bnb = body["symbol"].as_str() == Some("BNB");
        if pre_sale <= 0.0 || !symbol_is_bnb {
            return Ok(launch_fee);
        }
        let fee_rate = manager
            ._tradingFeeRate()
            .call()
            .await
            .map_err(|error| SdkError::Contract(error.to_string()))?;
        let presale_wei = bnb_to_wei_lossy(pre_sale);
        Ok(launch_fee
            + presale_wei
            + (presale_wei * fee_rate / alloy::primitives::U256::from(10_000)))
    }

    async fn get_api_data<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self.http.get(self.api_url(path)).send().await?;
        self.decode_envelope(response).await
    }

    async fn post_api_data<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let response = self.http.post(self.api_url(path)).json(body).send().await?;
        self.decode_envelope(response).await
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
            .await?;
        self.decode_envelope(response).await
    }

    async fn get_raw(&self, path: &str) -> Result<Value> {
        let response = self.http.get(self.api_url(path)).send().await?;
        Ok(response.error_for_status()?.json::<Value>().await?)
    }

    async fn post_raw<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<Value> {
        let response = self.http.post(self.api_url(path)).json(body).send().await?;
        Ok(response.error_for_status()?.json::<Value>().await?)
    }

    async fn decode_envelope<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let response = response.error_for_status()?;
        let text = response.text().await?;
        let envelope: ApiEnvelope<T> = serde_json::from_str(&text)?;
        if !envelope.is_success() {
            return Err(SdkError::Api {
                code: envelope.code_string(),
                body: text,
            });
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

fn epoch_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}
