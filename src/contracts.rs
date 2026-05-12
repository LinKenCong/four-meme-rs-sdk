//! Contract bindings for the Four.meme and EIP-8004 interfaces used by the SDK.
//!
//! These ABI fragments are maintained from the public BSC contracts and intentionally
//! contain only the calls/events this crate uses. Update `tests/abi_snapshots.rs`
//! when an upstream ABI change is verified.

use alloy::sol;

sol! {
    #[sol(rpc)]
    interface TokenManagerHelper3 {
        function getTokenInfo(address token) external view returns (
            uint256 version,
            address tokenManager,
            address quote,
            uint256 lastPrice,
            uint256 tradingFeeRate,
            uint256 minTradingFee,
            uint256 launchTime,
            uint256 offers,
            uint256 maxOffers,
            uint256 funds,
            uint256 maxFunds,
            bool liquidityAdded
        );

        function tryBuy(address token, uint256 amount, uint256 funds) external view returns (
            address tokenManager,
            address quote,
            uint256 estimatedAmount,
            uint256 estimatedCost,
            uint256 estimatedFee,
            uint256 amountMsgValue,
            uint256 amountApproval,
            uint256 amountFunds
        );

        function trySell(address token, uint256 amount) external view returns (
            address tokenManager,
            address quote,
            uint256 funds,
            uint256 fee
        );
    }

    #[sol(rpc)]
    interface TokenManager2 {
        function _launchFee() external view returns (uint256);
        function _tradingFeeRate() external view returns (uint256);
        function createToken(bytes args, bytes signature) external payable;
        function buyToken(address token, uint256 amount, uint256 maxFunds) external payable;
        function buyTokenAMAP(address token, uint256 funds, uint256 minAmount) external payable;
        function sellToken(address token, uint256 amount) external;
        function sellToken(uint256 origin, address token, uint256 amount, uint256 minFunds) external;

        event TokenCreate(address creator, address token, uint256 requestId, string name, string symbol, uint256 totalSupply, uint256 launchTime, uint256 launchFee);
        event TokenPurchase(address token, address account, uint256 price, uint256 amount, uint256 cost, uint256 fee, uint256 offers, uint256 funds);
        event TokenSale(address token, address account, uint256 price, uint256 amount, uint256 cost, uint256 fee, uint256 offers, uint256 funds);
        event LiquidityAdded(address base, uint256 offers, address quote, uint256 funds);
    }

    #[sol(rpc)]
    interface TaxToken {
        function feeRate() external view returns (uint256);
        function rateFounder() external view returns (uint256);
        function rateHolder() external view returns (uint256);
        function rateBurn() external view returns (uint256);
        function rateLiquidity() external view returns (uint256);
        function minDispatch() external view returns (uint256);
        function minShare() external view returns (uint256);
        function quote() external view returns (address);
        function founder() external view returns (address);
    }

    #[sol(rpc)]
    interface Erc20 {
        function approve(address spender, uint256 amount) external returns (bool);
        function transfer(address to, uint256 amount) external returns (bool);
    }

    #[sol(rpc)]
    interface Eip8004Nft {
        function balanceOf(address owner) external view returns (uint256);
        function register(string agentURI) external returns (uint256 agentId);
        event Registered(uint256 indexed agentId, string agentURI, address indexed owner);
    }
}
