use alloy::primitives::{Address, B256, U256};
use alloy::providers::Provider;

use crate::client::FourMemeSdk;
use crate::contracts::{Erc20, TaxToken, TokenManager2, TokenManagerHelper3};
use crate::error::{Result, SdkError};
use crate::types::{
    Asset, BuyMode, BuyQuote, CreateTokenApiOutput, SellQuote, TaxTokenInfo, TokenInfo,
};
use crate::utils::{normalize_hex_or_base64, optional_non_zero};
use crate::wallet::signer_from_private_key;

impl FourMemeSdk {
    pub async fn get_token_info(&self, token: Address) -> Result<TokenInfo> {
        let helper = TokenManagerHelper3::new(
            self.config.addresses.token_manager_helper3,
            self.provider.clone(),
        );
        let result = helper
            .getTokenInfo(token)
            .call()
            .await
            .map_err(|error| SdkError::rpc_provider("get token info", error))?;
        Ok(TokenInfo {
            version: result.version.to::<u64>(),
            token_manager: result.tokenManager,
            quote: optional_non_zero(result.quote),
            last_price: result.lastPrice,
            trading_fee_rate: result.tradingFeeRate.to::<u64>() as f64 / 10_000.0,
            min_trading_fee: result.minTradingFee,
            launch_time: result.launchTime.to::<u64>(),
            offers: result.offers,
            max_offers: result.maxOffers,
            funds: result.funds,
            max_funds: result.maxFunds,
            liquidity_added: result.liquidityAdded,
        })
    }

    pub async fn quote_buy(&self, token: Address, amount: U256, funds: U256) -> Result<BuyQuote> {
        let helper = TokenManagerHelper3::new(
            self.config.addresses.token_manager_helper3,
            self.provider.clone(),
        );
        let result = helper
            .tryBuy(token, amount, funds)
            .call()
            .await
            .map_err(|error| SdkError::rpc_provider("quote buy", error))?;
        Ok(BuyQuote {
            token_manager: result.tokenManager,
            quote: optional_non_zero(result.quote),
            estimated_amount: result.estimatedAmount,
            estimated_cost: result.estimatedCost,
            estimated_fee: result.estimatedFee,
            amount_msg_value: result.amountMsgValue,
            amount_approval: result.amountApproval,
            amount_funds: result.amountFunds,
        })
    }

    pub async fn quote_sell(&self, token: Address, amount: U256) -> Result<SellQuote> {
        let helper = TokenManagerHelper3::new(
            self.config.addresses.token_manager_helper3,
            self.provider.clone(),
        );
        let result = helper
            .trySell(token, amount)
            .call()
            .await
            .map_err(|error| SdkError::rpc_provider("quote sell", error))?;
        Ok(SellQuote {
            token_manager: result.tokenManager,
            quote: optional_non_zero(result.quote),
            funds: result.funds,
            fee: result.fee,
        })
    }

    pub async fn get_tax_token_info(&self, token: Address) -> Result<TaxTokenInfo> {
        let contract = TaxToken::new(token, self.provider.clone());
        let fee_rate = contract
            .feeRate()
            .call()
            .await
            .map_err(rpc_provider_error)?;
        let rate_founder = contract
            .rateFounder()
            .call()
            .await
            .map_err(rpc_provider_error)?;
        let rate_holder = contract
            .rateHolder()
            .call()
            .await
            .map_err(rpc_provider_error)?;
        let rate_burn = contract
            .rateBurn()
            .call()
            .await
            .map_err(rpc_provider_error)?;
        let rate_liquidity = contract
            .rateLiquidity()
            .call()
            .await
            .map_err(rpc_provider_error)?;
        let min_dispatch = contract
            .minDispatch()
            .call()
            .await
            .map_err(rpc_provider_error)?;
        let min_share = contract
            .minShare()
            .call()
            .await
            .map_err(rpc_provider_error)?;
        let quote = contract.quote().call().await.map_err(rpc_provider_error)?;
        let founder = contract
            .founder()
            .call()
            .await
            .map_err(rpc_provider_error)?;
        Ok(TaxTokenInfo {
            fee_rate_bps: fee_rate.to::<u64>(),
            fee_rate_percent: fee_rate.to::<u64>() as f64 / 100.0,
            rate_founder: rate_founder.to::<u64>(),
            rate_holder: rate_holder.to::<u64>(),
            rate_burn: rate_burn.to::<u64>(),
            rate_liquidity: rate_liquidity.to::<u64>(),
            min_dispatch,
            min_share,
            quote: optional_non_zero(quote),
            founder: optional_non_zero(founder),
        })
    }

    pub async fn submit_create_token(
        &self,
        private_key: impl AsRef<str>,
        create_arg: impl AsRef<str>,
        signature: impl AsRef<str>,
        value: U256,
    ) -> Result<B256> {
        let signer = signer_from_private_key(private_key)?;
        let provider = self.signer_provider(signer)?;
        let manager = TokenManager2::new(self.config.addresses.token_manager2, provider);
        let create_arg = normalize_hex_or_base64(create_arg)?;
        let signature = normalize_hex_or_base64(signature)?;
        let receipt = manager
            .createToken(create_arg, signature)
            .value(value)
            .send()
            .await
            .map_err(contract_error)?
            .get_receipt()
            .await
            .map_err(contract_error)?;
        Ok(receipt.transaction_hash)
    }

    pub async fn submit_prepared_create_token(
        &self,
        private_key: impl AsRef<str>,
        prepared: &CreateTokenApiOutput,
    ) -> Result<B256> {
        let value = prepared.creation_fee_wei.parse::<U256>().map_err(|_| {
            SdkError::validation(
                "creation_fee_wei",
                format!("invalid amount `{}`", prepared.creation_fee_wei),
            )
        })?;
        self.submit_create_token(
            private_key,
            &prepared.create_arg,
            &prepared.signature,
            value,
        )
        .await
    }

    pub async fn execute_buy(
        &self,
        private_key: impl AsRef<str>,
        token: Address,
        mode: BuyMode,
    ) -> Result<B256> {
        let token_info = self.get_token_info(token).await?;
        if token_info.version != 2 {
            return Err(SdkError::validation(
                "token_version",
                format!(
                    "unsupported token version {}; expected 2",
                    token_info.version
                ),
            ));
        }
        let (amount, funds) = match mode {
            BuyMode::FixedAmount { amount, .. } => (amount, U256::ZERO),
            BuyMode::FixedFunds { funds, .. } => (U256::ZERO, funds),
        };
        let quote = self.quote_buy(token, amount, funds).await?;
        let signer = signer_from_private_key(private_key)?;
        let provider = self.signer_provider(signer)?;
        if let Some(quote_token) = quote.quote.filter(|_| quote.amount_approval > U256::ZERO) {
            let erc20 = Erc20::new(quote_token, provider.clone());
            erc20
                .approve(token_info.token_manager, quote.amount_approval)
                .send()
                .await
                .map_err(contract_error)?
                .get_receipt()
                .await
                .map_err(contract_error)?;
        }
        let manager = TokenManager2::new(token_info.token_manager, provider);
        let pending = match mode {
            BuyMode::FixedAmount { amount, max_funds } => manager
                .buyToken(token, amount, max_funds)
                .value(quote.amount_msg_value)
                .send()
                .await
                .map_err(contract_error)?,
            BuyMode::FixedFunds { funds, min_amount } => manager
                .buyTokenAMAP(token, funds, min_amount)
                .value(quote.amount_msg_value)
                .send()
                .await
                .map_err(contract_error)?,
        };
        Ok(pending
            .get_receipt()
            .await
            .map_err(contract_error)?
            .transaction_hash)
    }

    pub async fn execute_sell(
        &self,
        private_key: impl AsRef<str>,
        token: Address,
        amount: U256,
        min_funds: Option<U256>,
    ) -> Result<B256> {
        if amount == U256::ZERO {
            return Err(SdkError::validation(
                "amount",
                format!("invalid amount `{amount}`"),
            ));
        }
        let token_info = self.get_token_info(token).await?;
        let signer = signer_from_private_key(private_key)?;
        let provider = self.signer_provider(signer)?;
        let token_contract = Erc20::new(token, provider.clone());
        token_contract
            .approve(token_info.token_manager, amount)
            .send()
            .await
            .map_err(contract_error)?
            .get_receipt()
            .await
            .map_err(contract_error)?;
        let manager = TokenManager2::new(token_info.token_manager, provider);
        let pending = if let Some(min_funds) = min_funds {
            manager
                .sellToken_1(U256::ZERO, token, amount, min_funds)
                .send()
                .await
                .map_err(contract_error)?
        } else {
            manager
                .sellToken_0(token, amount)
                .send()
                .await
                .map_err(contract_error)?
        };
        Ok(pending
            .get_receipt()
            .await
            .map_err(contract_error)?
            .transaction_hash)
    }

    pub async fn send_asset(
        &self,
        private_key: impl AsRef<str>,
        to: Address,
        amount: U256,
        asset: Asset,
    ) -> Result<B256> {
        if amount == U256::ZERO {
            return Err(SdkError::validation(
                "amount",
                format!("invalid amount `{amount}`"),
            ));
        }
        let signer = signer_from_private_key(private_key)?;
        let provider = self.signer_provider(signer)?;
        match asset {
            Asset::Native => {
                let tx = alloy::rpc::types::TransactionRequest::default()
                    .to(to)
                    .value(amount);
                Ok(provider
                    .send_transaction(tx)
                    .await
                    .map_err(contract_error)?
                    .get_receipt()
                    .await
                    .map_err(contract_error)?
                    .transaction_hash)
            }
            Asset::Erc20(token) => {
                let erc20 = Erc20::new(token, provider);
                Ok(erc20
                    .transfer(to, amount)
                    .send()
                    .await
                    .map_err(contract_error)?
                    .get_receipt()
                    .await
                    .map_err(contract_error)?
                    .transaction_hash)
            }
        }
    }
}

fn contract_error(error: impl std::fmt::Display) -> SdkError {
    SdkError::transaction_failed("transaction submission", error)
}

fn rpc_provider_error(error: impl std::fmt::Display) -> SdkError {
    SdkError::rpc_provider("contract call", error)
}
