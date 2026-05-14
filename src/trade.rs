//! Contract read/write helpers for trading, transfers, and tax-token inspection.
//!
//! Quote and planning methods are read-only. Execution methods submit transactions and return only
//! after receipt status validation.

use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use alloy::sol_types::SolCall;

use crate::api::encode_create_token_calldata;
use crate::client::FourMemeSdk;
use crate::contracts::{Erc20, TaxToken, TokenManager2, TokenManagerHelper3};
use crate::error::{Result, SdkError};
use crate::receipt::wait_for_confirmation;
use crate::types::{
    Asset, BuyExecutionPlan, BuyExecutionResult, BuyMode, BuyPlan, BuyQuote, ConfirmedReceipt,
    CreateTokenApiOutput, SellExecutionPlan, SellExecutionResult, SellPlan, SellQuote,
    TaxTokenInfo, TokenInfo, TradeApproval, TradeApprovalReceipt, TradeExecutionReceipt,
};
use crate::utils::{normalize_hex_or_base64, optional_non_zero};
use crate::wallet::signer_from_private_key;

impl TradeApproval {
    /// Returns the canonical ERC-20 `approve(address,uint256)` calldata for this approval.
    pub fn expected_calldata(&self) -> Bytes {
        encode_approval_calldata(self.spender, self.amount)
    }
}

impl BuyExecutionPlan {
    /// Returns the canonical TokenManager2 buy calldata for this execution plan.
    pub fn expected_calldata(&self) -> Bytes {
        match self {
            Self::FixedAmount {
                token,
                amount,
                max_funds,
                ..
            } => encode_buy_token_calldata(*token, *amount, *max_funds),
            Self::FixedFunds {
                token,
                funds,
                min_amount,
                ..
            } => encode_buy_token_amap_calldata(*token, *funds, *min_amount),
        }
    }
}

impl SellExecutionPlan {
    /// Returns the canonical TokenManager2 sell calldata for this execution plan.
    pub fn expected_calldata(&self) -> Bytes {
        encode_sell_token_calldata(self.token, self.amount, self.min_funds)
    }
}

/// Encodes ERC-20 `approve(address,uint256)` calldata.
pub fn encode_approval_calldata(spender: Address, amount: U256) -> Bytes {
    Erc20::approveCall { spender, amount }.abi_encode().into()
}

/// Encodes TokenManager2 `buyToken(address,uint256,uint256)` calldata.
pub fn encode_buy_token_calldata(token: Address, amount: U256, max_funds: U256) -> Bytes {
    TokenManager2::buyTokenCall {
        token,
        amount,
        maxFunds: max_funds,
    }
    .abi_encode()
    .into()
}

/// Encodes TokenManager2 `buyTokenAMAP(address,uint256,uint256)` calldata.
pub fn encode_buy_token_amap_calldata(token: Address, funds: U256, min_amount: U256) -> Bytes {
    TokenManager2::buyTokenAMAPCall {
        token,
        funds,
        minAmount: min_amount,
    }
    .abi_encode()
    .into()
}

/// Encodes the TokenManager2 sell calldata used by `SellExecutionPlan`.
pub fn encode_sell_token_calldata(token: Address, amount: U256, min_funds: Option<U256>) -> Bytes {
    match min_funds {
        Some(min_funds) => TokenManager2::sellToken_1Call {
            origin: U256::ZERO,
            token,
            amount,
            minFunds: min_funds,
        }
        .abi_encode()
        .into(),
        None => TokenManager2::sellToken_0Call { token, amount }
            .abi_encode()
            .into(),
    }
}

impl FourMemeSdk {
    /// Reads TokenManagerHelper3 token state used by quote and trading flows.
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

    /// Quotes a buy by desired token amount or fixed quote-token/native funds.
    ///
    /// Pass exactly one non-zero input and set the unused value to `U256::ZERO`.
    pub async fn quote_buy(&self, token: Address, amount: U256, funds: U256) -> Result<BuyQuote> {
        validate_quote_inputs(amount, funds)?;
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

    /// Builds a quote-first buy plan without submitting approval or buy transactions.
    pub async fn plan_buy(&self, token: Address, mode: BuyMode) -> Result<BuyPlan> {
        mode.validate()?;
        let token_info = self.get_supported_trade_token_info(token).await?;
        let (amount, funds) = mode.quote_inputs();
        let quote = self.quote_buy(token, amount, funds).await?;
        ensure_quote_matches_manager(quote.token_manager, token_info.token_manager)?;
        let approval = buy_approval(&quote, token_info.token_manager);
        let execution = buy_execution_plan(token, mode, quote.amount_msg_value);
        Ok(BuyPlan {
            token,
            token_manager: token_info.token_manager,
            mode,
            quote,
            approval,
            execution,
        })
    }

    /// Quotes the funds and fee returned by selling a token amount.
    pub async fn quote_sell(&self, token: Address, amount: U256) -> Result<SellQuote> {
        validate_non_zero_amount("amount", amount)?;
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

    /// Builds a sell plan without submitting approval or sell transactions.
    pub async fn plan_sell(
        &self,
        token: Address,
        amount: U256,
        min_funds: Option<U256>,
    ) -> Result<SellPlan> {
        validate_non_zero_amount("amount", amount)?;
        validate_optional_non_zero_amount("min_funds", min_funds)?;
        let token_info = self.get_supported_trade_token_info(token).await?;
        let quote = self.quote_sell(token, amount).await?;
        ensure_quote_matches_manager(quote.token_manager, token_info.token_manager)?;
        Ok(SellPlan {
            token,
            token_manager: token_info.token_manager,
            quote,
            approval: TradeApproval {
                token,
                spender: token_info.token_manager,
                amount,
                calldata: encode_approval_calldata(token_info.token_manager, amount),
            },
            execution: SellExecutionPlan {
                token,
                value: U256::ZERO,
                amount,
                min_funds,
                calldata: encode_sell_token_calldata(token, amount, min_funds),
            },
        })
    }

    /// Reads tax-token fee and distribution configuration.
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

    /// Submits reviewed `createToken` calldata and value to TokenManager2.
    pub async fn submit_create_token(
        &self,
        private_key: impl AsRef<str>,
        create_arg: impl AsRef<str>,
        signature: impl AsRef<str>,
        value: U256,
    ) -> Result<ConfirmedReceipt> {
        let calldata = encode_create_token_calldata(create_arg, signature)?;
        self.submit_contract_calldata(
            private_key,
            self.config.addresses.token_manager2,
            value,
            calldata,
        )
        .await
    }

    /// Submits a previously prepared token creation payload.
    pub async fn submit_prepared_create_token(
        &self,
        private_key: impl AsRef<str>,
        prepared: &CreateTokenApiOutput,
    ) -> Result<ConfirmedReceipt> {
        let value = prepared.creation_fee_wei.parse::<U256>().map_err(|_| {
            SdkError::validation(
                "creation_fee_wei",
                format!("invalid amount `{}`", prepared.creation_fee_wei),
            )
        })?;
        let calldata = resolved_string_calldata(
            "create_token.calldata",
            &prepared.calldata,
            prepared.expected_calldata()?,
        )?;
        self.submit_contract_calldata(
            private_key,
            self.config.addresses.token_manager2,
            value,
            calldata,
        )
        .await
    }

    /// Compatibility buy entry point that plans, approves when required, executes, and confirms.
    pub async fn execute_buy(
        &self,
        private_key: impl AsRef<str>,
        token: Address,
        mode: BuyMode,
    ) -> Result<ConfirmedReceipt> {
        let result = self.execute_buy_with_plan(private_key, token, mode).await?;
        Ok(result.execution.receipt)
    }

    /// Runs a full buy workflow and returns both the plan and confirmed receipts.
    pub async fn execute_buy_with_plan(
        &self,
        private_key: impl AsRef<str>,
        token: Address,
        mode: BuyMode,
    ) -> Result<BuyExecutionResult> {
        let private_key = private_key.as_ref();
        let plan = self.plan_buy(token, mode).await?;
        let approval = self.approve_buy(private_key, &plan).await?;
        let execution = self.execute_buy_plan(private_key, &plan).await?;
        Ok(BuyExecutionResult {
            plan,
            approval,
            execution,
        })
    }

    /// Submits the ERC-20 quote-token approval required by a buy plan, when any.
    pub async fn approve_buy(
        &self,
        private_key: impl AsRef<str>,
        plan: &BuyPlan,
    ) -> Result<Option<TradeApprovalReceipt>> {
        let Some(approval) = plan.approval.as_ref() else {
            return Ok(None);
        };
        let receipt = self.submit_approval(private_key, approval).await?;
        Ok(Some(TradeApprovalReceipt {
            approval: approval.clone(),
            receipt,
        }))
    }

    /// Executes an already reviewed buy plan without submitting its approval.
    pub async fn execute_buy_plan(
        &self,
        private_key: impl AsRef<str>,
        plan: &BuyPlan,
    ) -> Result<TradeExecutionReceipt> {
        let execution = &plan.execution;
        let calldata = resolved_bytes_calldata(
            "buy.execution.calldata",
            match execution {
                BuyExecutionPlan::FixedAmount { calldata, .. }
                | BuyExecutionPlan::FixedFunds { calldata, .. } => calldata,
            },
            execution.expected_calldata(),
        )?;
        let receipt = self
            .submit_contract_calldata(private_key, plan.token_manager, execution.value(), calldata)
            .await?;
        Ok(TradeExecutionReceipt {
            token: plan.token,
            token_manager: plan.token_manager,
            value: execution.value(),
            receipt,
        })
    }

    /// Compatibility sell entry point that plans, approves, executes, and confirms.
    pub async fn execute_sell(
        &self,
        private_key: impl AsRef<str>,
        token: Address,
        amount: U256,
        min_funds: Option<U256>,
    ) -> Result<ConfirmedReceipt> {
        let result = self
            .execute_sell_with_plan(private_key, token, amount, min_funds)
            .await?;
        Ok(result.execution.receipt)
    }

    /// Runs a full sell workflow and returns the plan plus confirmed approval/execution receipts.
    pub async fn execute_sell_with_plan(
        &self,
        private_key: impl AsRef<str>,
        token: Address,
        amount: U256,
        min_funds: Option<U256>,
    ) -> Result<SellExecutionResult> {
        let private_key = private_key.as_ref();
        let plan = self.plan_sell(token, amount, min_funds).await?;
        let approval = self.approve_sell(private_key, &plan).await?;
        let execution = self.execute_sell_plan(private_key, &plan).await?;
        Ok(SellExecutionResult {
            plan,
            approval,
            execution,
        })
    }

    /// Submits the ERC-20 token approval required by a sell plan.
    pub async fn approve_sell(
        &self,
        private_key: impl AsRef<str>,
        plan: &SellPlan,
    ) -> Result<TradeApprovalReceipt> {
        let approval = &plan.approval;
        let receipt = self.submit_approval(private_key, approval).await?;
        Ok(TradeApprovalReceipt {
            approval: approval.clone(),
            receipt,
        })
    }

    /// Executes an already reviewed sell plan without submitting its approval.
    pub async fn execute_sell_plan(
        &self,
        private_key: impl AsRef<str>,
        plan: &SellPlan,
    ) -> Result<TradeExecutionReceipt> {
        let execution = &plan.execution;
        let calldata = resolved_bytes_calldata(
            "sell.execution.calldata",
            &execution.calldata,
            execution.expected_calldata(),
        )?;
        let receipt = self
            .submit_contract_calldata(private_key, plan.token_manager, execution.value, calldata)
            .await?;
        Ok(TradeExecutionReceipt {
            token: plan.token,
            token_manager: plan.token_manager,
            value: execution.value,
            receipt,
        })
    }

    async fn get_supported_trade_token_info(&self, token: Address) -> Result<TokenInfo> {
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
        Ok(token_info)
    }

    async fn submit_approval(
        &self,
        private_key: impl AsRef<str>,
        approval: &TradeApproval,
    ) -> Result<ConfirmedReceipt> {
        let calldata = resolved_bytes_calldata(
            "approval.calldata",
            &approval.calldata,
            approval.expected_calldata(),
        )?;
        self.submit_contract_calldata(private_key, approval.token, U256::ZERO, calldata)
            .await
    }

    async fn submit_contract_calldata(
        &self,
        private_key: impl AsRef<str>,
        to: Address,
        value: U256,
        calldata: Bytes,
    ) -> Result<ConfirmedReceipt> {
        let signer = signer_from_private_key(private_key)?;
        let provider = self.signer_provider(signer)?;
        let tx = TransactionRequest::default()
            .to(to)
            .value(value)
            .input(TransactionInput::new(calldata));
        let pending = provider
            .send_transaction(tx)
            .await
            .map_err(contract_error)?;
        wait_for_confirmation(pending).await
    }

    /// Sends native BNB or an ERC-20 transfer and validates the receipt status.
    pub async fn send_asset(
        &self,
        private_key: impl AsRef<str>,
        to: Address,
        amount: U256,
        asset: Asset,
    ) -> Result<ConfirmedReceipt> {
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
                let pending = provider
                    .send_transaction(tx)
                    .await
                    .map_err(contract_error)?;
                wait_for_confirmation(pending).await
            }
            Asset::Erc20(token) => {
                let erc20 = Erc20::new(token, provider);
                let pending = erc20
                    .transfer(to, amount)
                    .send()
                    .await
                    .map_err(contract_error)?;
                wait_for_confirmation(pending).await
            }
        }
    }
}

fn validate_quote_inputs(amount: U256, funds: U256) -> Result<()> {
    match (amount > U256::ZERO, funds > U256::ZERO) {
        (true, false) | (false, true) => Ok(()),
        (false, false) => Err(SdkError::validation(
            "amount_or_funds",
            "one input must be greater than zero",
        )),
        (true, true) => Err(SdkError::validation(
            "amount_or_funds",
            "only one input can be greater than zero",
        )),
    }
}

fn validate_non_zero_amount(field: &'static str, amount: U256) -> Result<()> {
    if amount == U256::ZERO {
        return Err(SdkError::validation(field, "must be greater than zero"));
    }
    Ok(())
}

fn validate_optional_non_zero_amount(field: &'static str, amount: Option<U256>) -> Result<()> {
    if amount == Some(U256::ZERO) {
        return Err(SdkError::validation(field, "must be greater than zero"));
    }
    Ok(())
}

fn ensure_quote_matches_manager(quoted: Address, expected: Address) -> Result<()> {
    if quoted != expected {
        return Err(SdkError::validation(
            "token_manager",
            "quote returned a different token manager",
        ));
    }
    Ok(())
}

fn buy_approval(quote: &BuyQuote, spender: Address) -> Option<TradeApproval> {
    quote
        .quote
        .filter(|_| quote.amount_approval > U256::ZERO)
        .map(|token| TradeApproval {
            token,
            spender,
            amount: quote.amount_approval,
            calldata: encode_approval_calldata(spender, quote.amount_approval),
        })
}

fn buy_execution_plan(token: Address, mode: BuyMode, value: U256) -> BuyExecutionPlan {
    match mode {
        BuyMode::FixedAmount { amount, max_funds } => BuyExecutionPlan::FixedAmount {
            token,
            value,
            amount,
            max_funds,
            calldata: encode_buy_token_calldata(token, amount, max_funds),
        },
        BuyMode::FixedFunds { funds, min_amount } => BuyExecutionPlan::FixedFunds {
            token,
            value,
            funds,
            min_amount,
            calldata: encode_buy_token_amap_calldata(token, funds, min_amount),
        },
    }
}

fn resolved_bytes_calldata(field: &'static str, actual: &Bytes, expected: Bytes) -> Result<Bytes> {
    if actual.is_empty() {
        return Ok(expected);
    }
    if *actual != expected {
        return Err(SdkError::validation(
            field,
            "calldata does not match structured plan fields",
        ));
    }
    Ok(actual.clone())
}

fn resolved_string_calldata(field: &'static str, actual: &str, expected: Bytes) -> Result<Bytes> {
    if actual.trim().is_empty() {
        return Ok(expected);
    }
    let actual = normalize_hex_or_base64(actual)?;
    if actual != expected {
        return Err(SdkError::validation(
            field,
            "calldata does not match prepared token payload",
        ));
    }
    Ok(actual)
}

fn contract_error(error: impl std::fmt::Display) -> SdkError {
    SdkError::transaction_failed("transaction submission", error)
}

fn rpc_provider_error(error: impl std::fmt::Display) -> SdkError {
    SdkError::rpc_provider("contract call", error)
}
