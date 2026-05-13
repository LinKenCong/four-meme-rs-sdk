use alloy::primitives::{Address, U256};
use alloy::signers::Signer;
use reqwest::multipart::{Form, Part};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::client::FourMemeSdk;
use crate::contracts::TokenManager2;
use crate::error::{RedactedContext, Result, SdkError};
use crate::types::{
    ApiEnvelope, CreateTokenApiOutput, CreateTokenImage, CreateTokenRequest, PublicConfig,
    RaisedToken, RankingRequest, TokenDetail, TokenRankingResponse, TokenSearchRequest,
    TokenSearchResponse,
};
use crate::utils::{normalize_hex_or_base64, parse_address, parse_bnb_to_wei};
use crate::wallet::signer_from_private_key;

impl FourMemeSdk {
    pub async fn public_config(&self) -> Result<PublicConfig> {
        self.get_api_data("/public/config").await
    }

    pub async fn token_detail(&self, address: Address) -> Result<TokenDetail> {
        let url = format!("/private/token/get/v2?address={address}");
        self.get_api_data(&url).await
    }

    pub async fn token_detail_raw(&self, address: Address) -> Result<Value> {
        let url = format!("/private/token/get/v2?address={address}");
        self.get_raw(&url).await
    }

    pub async fn token_search(&self, request: &TokenSearchRequest) -> Result<TokenSearchResponse> {
        self.post_api_data("/public/token/search", request).await
    }

    pub async fn token_search_raw(&self, request: &TokenSearchRequest) -> Result<Value> {
        self.post_raw("/public/token/search", request).await
    }

    pub async fn token_rankings(&self, request: &RankingRequest) -> Result<TokenRankingResponse> {
        self.post_api_data("/public/token/ranking", request).await
    }

    pub async fn token_rankings_raw(&self, request: &RankingRequest) -> Result<Value> {
        self.post_raw("/public/token/ranking", request).await
    }

    pub async fn prepare_create_token(
        &self,
        private_key: impl AsRef<str>,
        request: CreateTokenRequest,
    ) -> Result<CreateTokenApiOutput> {
        request.validate()?;

        let signer = signer_from_private_key(private_key)?;
        let address = signer.address();
        let access_token = self.login(address, &signer).await?;
        let img_url = match &request.image {
            CreateTokenImage::File {
                file_name,
                bytes,
                content_type,
            } => {
                self.upload_token_image(
                    &access_token,
                    file_name,
                    bytes.clone(),
                    content_type.as_deref(),
                )
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

    pub async fn login_with_signer(&self, signer: &(impl Signer + Sync)) -> Result<String> {
        self.login(signer.address(), signer).await
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
        content_type: Option<&str>,
    ) -> Result<String> {
        let mut part = Part::bytes(bytes).file_name(file_name.to_string());
        if let Some(content_type) = content_type {
            part = part
                .mime_str(content_type)
                .map_err(|error| SdkError::InvalidTokenImage(error.to_string()))?;
        }
        let form = Form::new().part("file", part);
        let response = self
            .send_api_request(
                self.http
                    .post(self.api_url("/private/token/upload"))
                    .header("meme-web-access", access_token)
                    .multipart(form),
            )
            .await?;
        self.decode_envelope(response).await
    }

    async fn preferred_raised_token(&self) -> Result<RaisedToken> {
        let config = self.public_config().await?;
        let tokens = config.raised_tokens;
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
        let metadata = RaisedTokenCreateMetadata::try_from(&raised_token)?;
        let mut body = json!({
            "name": request.name,
            "shortName": request.short_name,
            "desc": request.desc,
            "totalSupply": metadata.total_supply,
            "raisedAmount": metadata.raised_amount,
            "saleRate": metadata.sale_rate,
            "reserveRate": 0,
            "imgUrl": img_url,
            "raisedToken": raised_token,
            "launchTime": epoch_millis(),
            "funGroup": false,
            "label": request.label.as_api_str(),
            "lpTradingFee": 0.0025,
            "preSale": request.pre_sale,
            "clickFun": false,
            "symbol": metadata.symbol,
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
        let presale_wei = create_presale_wei(body)?;
        if presale_wei == U256::ZERO || body["symbol"].as_str() != Some("BNB") {
            return Ok(launch_fee);
        }
        let fee_rate = manager
            ._tradingFeeRate()
            .call()
            .await
            .map_err(|error| SdkError::Contract(error.to_string()))?;
        calculate_creation_fee_wei(launch_fee, presale_wei, fee_rate)
    }

    async fn get_api_data<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self
            .send_api_request(self.http.get(self.api_url(path)))
            .await?;
        self.decode_envelope(response).await
    }

    async fn post_api_data<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let response = self
            .send_api_request(self.http.post(self.api_url(path)).json(body))
            .await?;
        self.decode_envelope(response).await
    }

    async fn post_api_data_with_access<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        access_token: &str,
        body: &B,
    ) -> Result<T> {
        let response = self
            .send_api_request(
                self.http
                    .post(self.api_url(path))
                    .header("meme-web-access", access_token)
                    .json(body),
            )
            .await?;
        self.decode_envelope(response).await
    }

    async fn get_raw(&self, path: &str) -> Result<Value> {
        let response = self
            .send_api_request(self.http.get(self.api_url(path)))
            .await?;
        Ok(response.error_for_status()?.json::<Value>().await?)
    }

    async fn post_raw<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<Value> {
        let response = self
            .send_api_request(self.http.post(self.api_url(path)).json(body))
            .await?;
        Ok(response.error_for_status()?.json::<Value>().await?)
    }

    async fn decode_envelope<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let response = response.error_for_status()?;
        let text = response.text().await?;
        let envelope: ApiEnvelope<T> = serde_json::from_str(&text)?;
        if !envelope.is_success() {
            return Err(SdkError::rest_business(
                envelope.code_string(),
                envelope.message_text(),
                RedactedContext::new([("response_body", text)]),
            ));
        }
        envelope.data.ok_or_else(|| {
            SdkError::serialization("api envelope", "success response is missing data")
        })
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTokenApiData {
    create_arg: String,
    signature: String,
}

struct RaisedTokenCreateMetadata {
    symbol: String,
    total_supply: f64,
    raised_amount: f64,
    sale_rate: f64,
}

impl TryFrom<&RaisedToken> for RaisedTokenCreateMetadata {
    type Error = SdkError;

    fn try_from(token: &RaisedToken) -> Result<Self> {
        validate_raised_token_address(token)?;
        Ok(Self {
            symbol: token.symbol.clone(),
            total_supply: required_number_field(token, "total_amount", &token.total_amount)?,
            raised_amount: required_number_field(token, "total_b_amount", &token.total_b_amount)?,
            sale_rate: required_number_field(token, "sale_rate", &token.sale_rate)?,
        })
    }
}

fn hex_string(bytes: alloy::primitives::Bytes) -> String {
    format!("0x{}", hex::encode(bytes))
}

fn create_presale_wei(body: &Value) -> Result<U256> {
    match body["preSale"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => parse_bnb_to_wei(value),
        None => Ok(U256::ZERO),
    }
}

fn calculate_creation_fee_wei(launch_fee: U256, presale_wei: U256, fee_rate: U256) -> Result<U256> {
    let trading_fee = presale_wei
        .checked_mul(fee_rate)
        .and_then(|value| value.checked_div(U256::from(10_000u16)))
        .ok_or_else(|| SdkError::InvalidAmount("creation fee overflow".to_string()))?;
    launch_fee
        .checked_add(presale_wei)
        .and_then(|value| value.checked_add(trading_fee))
        .ok_or_else(|| SdkError::InvalidAmount("creation fee overflow".to_string()))
}

fn required_number_field(
    token: &RaisedToken,
    field: &'static str,
    value: &Option<String>,
) -> Result<f64> {
    let Some(value) = value else {
        return Err(raised_token_error(token, field, "missing"));
    };
    let parsed = value.parse::<f64>().ok();
    match parsed {
        Some(number) if number.is_finite() && number > 0.0 => Ok(number),
        _ => Err(raised_token_error(
            token,
            field,
            "must be a positive number",
        )),
    }
}

fn validate_raised_token_address(token: &RaisedToken) -> Result<()> {
    if token.symbol.trim().is_empty() {
        return Err(raised_token_error(token, "symbol", "missing"));
    }
    let Some(address) = &token.symbol_address else {
        return Err(raised_token_error(token, "symbol_address", "missing"));
    };
    parse_address(address)
        .map(|_| ())
        .map_err(|_| raised_token_error(token, "symbol_address", "must be a valid EVM address"))
}

fn raised_token_error(token: &RaisedToken, field: &'static str, reason: &str) -> SdkError {
    SdkError::InvalidRaisedTokenField {
        field,
        reason: format!("{} for `{}`", reason, token.symbol),
    }
}

fn epoch_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}
