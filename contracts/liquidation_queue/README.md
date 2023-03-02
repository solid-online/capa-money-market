# Liquidation Queue

The Liquidation contract enables users to submit Cw20 stablecoin bids for a Cw20-compliant token. Bidders can submit a bid to one of the bid pools; each of the pools deposited funds are used to buy the liquidated collateral at different discount rates. There are 31 slots per collateral, from 0% to 30%; users can bid on one or more slots.
Upon execution of a bid, Cw20 tokens are sent to the bidder, while the bidder's cw20 stablecoins are sent to the repay address (if not specified, sent to message sender). A portion of the collateral value liquidated will be given to the address triggering the liquidation (liquidator_fee).

Additionally, the Liquidation contract serves as the point of calculation for partial collateral liquidations, where a loan position is liquidated until it reaches a safe borrow_amount / borrow_limit ratio. The required liquidation amount for each collateral is calculated based on the fed-in loan position's attributes and the state of the bid pools.
The oracle contract is responsible for providing the relevant Cw20 token prices. Price data from the Oracle contract are only valid for 60 seconds (price_timeframe). The Liquidation contract disables bid executions until new price data is fed in to the Oracle contract.


## InstantiateMsg

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub oracle_contract: String,
    pub stable_denom: String,
    pub safe_ratio: Decimal256,
    pub bid_fee: Decimal256,
    pub liquidator_fee: Decimal256,
    pub liquidation_threshold: Uint256,
    pub price_timeframe: u64,
    pub waiting_period: u64,
    pub overseer: String,
}
```

### Example

```
{
  "owner": "terra1..", 
  "oracle_contract": "terra1...", 
  "stable_denom": "uusd", 
  "safe_ratio": "0.8", 
  "bid_fee": "0.01", 
  "liquidator_fee": "0.01",
  "liquidation_threshold": "500", 
  "price_timeframe": 60,
  "waiting_period": 600,
  "overseer": "terra1..."
}
```

## ExecuteMsg

### Receive
Can be called during a CW20 token transfer when the Liquidation contract is the recipient. Allows the token transfer to execute a Receive Hook as a subsequent action within the same transaction.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive {
        sender: String, 
        amount: Uint128, 
        msg: Binary, 
    }
}
```

#### Example

```
{
  "receive": {
    "sender": "terra1...", 
    "amount": "10000000", 
    "msg": "eyAiZXhlY3V0ZV9tc2ciOiAiYmluYXJ5IiB9" 
  }
}
```

### UpdateConfig 
Updates the Liquidation contract's configuration. Can only be issued by the owner.
```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg { 
    UpdateConfig {
        owner: Option<String>,
        oracle_contract: Option<String>,
        safe_ratio: Option<Decimal256>,
        bid_fee: Option<Decimal256>,
        liquidator_fee: Option<Decimal256>,
        liquidation_threshold: Option<Uint256>,
        price_timeframe: Option<u64>,
        waiting_period: Option<u64>,
        overseer: Option<String>,
    }
}
```

#### Example

```
{
  "update_config": {
    "owner": "terra1...", 
    "oracle_contract": "terra1...", 
    "safe_ratio": "0.8", 
    "bid_fee": "0.01", 
    "liquidator_fee": "0.01",
    "liquidation_threshold": "200000000", 
    "price_timeframe": 60,
    "waiting_period": 600,
    "overseer": "terra1..."
  }
}
```

### WhitelistCollateral 
Whitelist a new collateral token. Can only be issued by the owner.
```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg { 
    WhitelistCollateral {
        collateral_token: String,
        bid_threshold: Uint256,
        max_slot: u8,
        premium_rate_per_slot: Decimal256,
    }
}
```

#### Example

```
{
  "whitelist_collateral": {
    "collateral_token": "terra1...", 
    "bid_threshold": "200000000", 
    "max_slot": 30,
    "premium_rate_per_slot": "0.01",
  }
}
```

### UpdateCollateralInfo 
Update a whitelisted collateral configuration. Can only be issued by the owner.
```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg { 
    UpdateCollateralInfo {
        collateral_token: String,
        bid_threshold: Option<Uint256>,
        max_slot: Option<u8>,
    }
}
```

#### Example

```
{
  "update_collateral_info": {
    "collateral_token": "terra1...", 
    "bid_threshold": "200000000", 
    "max_slot": 30,
  }
}
```



### ActivateBids
Activates the list of bids_idx for the specified collateral_token. Activates all bids with expired waiting_period of the bid_idx field is not filled.
```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ActivateBids {
        collateral_token: String,
        bids_idx: Option<Vec<Uint128>>, 
    }
}
```

#### Example

```
{
  "activate_bids": {
    "collateral_token": "terra1...",
    "bids_idx": ["123","231"], 
  }
}
```

### RetractBid
Withdraws specified amount of stablecoins from the specified bid_idx. Withdraws the entire remaining bid if the amount field is not filled.
```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    RetractBid {
        bid_idx: Uint128, 
        amount: Option<Uint256>, 
    }
}
```

#### Example

```
{
  "retract_bid": {
    "bid_idx": "123", 
    "amount": "100000000" 
  }
}
```


### ClaimLiquidations
Claim the liquidated collateral accrued by the specified list of bids_idx. Claims all the pending collateral from user submitted bids if the bids_idx field is not filled.
```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ClaimLiquidations {
        collateral_token: String,
        bids_idx: Option<Vec<Uint128>>,
    }
}
```

#### Example

```
{
  "claim_liquidations": {
    "collateral_token": "terra1...",
    "bids_idx": ["123","231"],  
  }
}
```

## Receive Hooks

Liquidates the sent collateral using the active bids on the bid pools, consuming bids with lower premium rates first. Can only be executed by a whitelisted collateral's Custody contract. Custody issues this message with fee_address as Overseer's contract address and repay_address as Market's contract address, where stablecoins from the bid is sent to the Market contract to repay a borrower's loan and fees are sent to the Overseer contract to be added to the interest buffer. The liquidator_fee is sent to the liquidator address that originally triggered the liquidation.

### ExecuteBid 
```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    ExecuteBid {
        liquidator: String, 
        fee_address: Option<String>, // Filled as Overseer contract's address
        repay_address: Option<String>, // Filled as Market contract's address
    }
}
```

#### Example

```
{
  "execute_bid": {
    "liquidator": "terra1...", 
    "fee_address": "terra1...", // Filled as Overseer contract's address
    "repay_address": "terra1..." // Filled as Market contract's address
  }
}
```


### SubmitBid

Submits a new bid for the specified Cw20 collateral to the specified premium slot. Requires Cw20 stablecoins to be sent beforehand. The premium rate on each slot can be calculated using the whitelisted collateral configuration (premium_slot * premium_rate_per_slot).

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SubmitBid {
        collateral_token: String, 
        premium_slot: u8, 
    }
}
```

#### Example

```
{
  "submit_bid": {
    "collateral_token": "terra1...", 
    "premium_slot": 3
  }
}
```

## QueryMsg

### Config
Gets the Liquidation Contract's configuration.
```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {}
}
```

#### Example

```
{
  "config": {}
}
```

### ConfigResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub oracle_contract: String,
    pub stable_denom: String,
    pub safe_ratio: Decimal256,
    pub bid_fee: Decimal256,
    pub liquidator_fee: Decimal256,
    pub liquidation_threshold: Uint256,
    pub price_timeframe: u64,
    pub waiting_period: u64,
    pub overseer: String,
}
```

#### Example

```
{
  "owner": "terra1...", 
  "oracle_contract": "terra1...", 
  "stable_denom": "uusd", 
  "safe_ratio": "0.8", 
  "bid_fee": "0.01",
  "liquidator_fee": "0.01", 
  "liquidation_threshold": "200000000", 
  "price_timeframe": 60,
  "waiting_period": 600,
  "overseer": "terra1..."
}
```

### CollateralInfo
Gets collateral specific configuration.
```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    CollateralInfo {
        collateral_token: String,
    }
}
```

#### Example

```
{
  "collateral_info": {
      "collateral_token": "terra1..."
  }
}
```

### CollateralInfoResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollateralInfoResponse {
    pub collateral_token: String,
    pub bid_threshold: Uint256,
    pub max_slot: u8,
    pub premium_rate_per_slot: Decimal256,
}
```

#### Example

```
{
  "collateral_token": "terra1...", 
  "bid_threshold": "5000000000000",
  "max_slot": 30,
  "premium_rate_per_slot": "0.01"
}
```

### LiquidationAmount
Gets the amount of collaterals that needs to be liquidated in order for the borrower's loan to reach safe_ratio, based on the fed in borrower's status and the state of all bid pools.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    LiquidationAmount {
        borrow_amount: Uint256, 
        borrow_limit: Uint256, 
        collaterals: TokensHuman, 
        collateral_prices: Vec<Decimal>, 
    }
}

pub type TokensHuman = Vec<(String, Uint256)>;
```

#### Example

```
{
  "liquidation_amount": {
    "borrow_amount": "10000000", 
    "borrow_limit": "10000000", 
    "collaterals": [
      ["terra1...", "100000000"], // (Cw20 contract address, Locked amount)
      ["terra1...", "100000000"] 
    ], 
    "collateral_prices": [
      "123.456789", // Price of collateral
      "123.456789" 
    ]
  }
}
```

### LiquidationAmountResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidationAmountResponse {
    pub collaterals: TokensHuman, 
}

pub type TokensHuman = Vec<(String, Uint256)>;
```

#### Example

```
{
  "collaterals": [
    ["terra1...", "100000000"], // (Cw20 Token address, Required liquidation amount to reach safe_ratio)
    ["terra1...", "100000000"] 
  ] 
}
```

### Bid
Gets information about the specified bid_idx.
```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Bid {
        bid_idx: Uint128, 
    }
}
```

#### Example

```
{
  "bid": {
    "bid_idx": "123", 
  }
}
```

### BidResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidResponse {
    pub idx: Uint128,
    pub collateral_token: String,
    pub premium_slot: u8,
    pub bidder: String,
    pub amount: Uint256,
    pub product_snapshot: Decimal256,
    pub sum_snapshot: Decimal256,
    pub pending_liquidated_collateral: Uint256,
    pub wait_end: Option<u64>,
    pub epoch_snapshot: Uint128,
    pub scale_snapshot: Uint128,
}
```

#### Example

```
{
  "idx": "123",
  "collateral_token": "terra1...", 
  "premium_slot": "10",
  "bidder": "terra1...", 
  "amount": "100000000", 
  "product_snapshot": "1.0",
  "sum_snapshot": "1.0",
  "pending_liquidated_collateral": "0",
  "wait_end": "1635228109",
  "epoch_snapshot": "0",
  "scale_snapshot": "0"
}
```


### BidsByUser
Gets information for all bids submitted by the specified bidder and collateral.
```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    BidsByUser {
        collateral_token: String,
        bidder: String, 
        start_after: Option<Uint128>, 
        limit: Option<u8>, 
    }
}
```

#### Example

```
{
  "bids_by_user": {
    "collateral_token": "terra1...",
    "bidder": "terra1...", 
    "start_after": "123", 
    "limit": 8 
  }
}
```


### BidsByUserResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidsResponse {
    pub bids: Vec<BidResponse>, 
}

pub struct BidResponse {
    pub idx: Uint128,
    pub collateral_token: String,
    pub premium_slot: u8,
    pub bidder: String,
    pub amount: Uint256,
    pub product_snapshot: Decimal256,
    pub sum_snapshot: Decimal256,
    pub pending_liquidated_collateral: Uint256,
    pub wait_end: Option<u64>,
    pub epoch_snapshot: Uint128,
    pub scale_snapshot: Uint128,
}
```

#### Example

```
{
  "bids": [
    {
      "idx": "12",
      "collateral_token": "terra1...", 
      "premium_slot": "10",
      "bidder": "terra1...", 
      "amount": "100000000", 
      "product_snapshot": "1.0",
      "sum_snapshot": "1.0",
      "pending_liquidated_collateral": "0",
      "wait_end": "1635228109",
      "epoch_snapshot": "0",
      "scale_snapshot": "0"
    }, 
    {
      "idx": "26",
      "collateral_token": "terra1...", 
      "premium_slot": "11",
      "bidder": "terra1...", 
      "amount": "200000000", 
      "product_snapshot": "1.0",
      "sum_snapshot": "1.0",
      "pending_liquidated_collateral": "0",
      "wait_end": "1635228109",
      "epoch_snapshot": "0",
      "scale_snapshot": "0"
    }
  ]
}
```


### BidPool

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    BidPool {
        collateral_token: String,
        bid_slot: u8,
    },
}
```

#### Example

```
{
  "bid_pool": {
    "collateral_token": "terra1...",
    "bid_slot": 10, 
  }
}
```


### BidPoolResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidPoolResponse {
    pub sum_snapshot: Decimal256,
    pub product_snapshot: Decimal256,
    pub total_bid_amount: Uint256,
    pub premium_rate: Decimal256,
    pub current_epoch: Uint128,
    pub current_scale: Uint128,
}
```

#### Example

```
{
  "sum_snapshot": "1.0",
  "product_snapshot": "1.0",
  "total_bid_amount": "1000000",
  "premium_rate": "0.1",
  "current_epoch": "0",
  "current_scale": "0"
}
```


### BidPoolsByCollateral

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    BidPoolsByCollateral {
        collateral_token: String,
        start_after: Option<u8>,
        limit: Option<u8>,
    },
}
```

#### Example

```
{
  "bid_pools_by_collateral": {
    "collateral_token": "terra1...", 
    "start_after": 1, 
    "limit": 10 
  }
}
```


### BidPoolsByCollateralResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidPoolsResponse {
    pub bid_pools: Vec<BidPoolResponse>,
}

pub struct BidPoolResponse {
    pub sum_snapshot: Decimal256,
    pub product_snapshot: Decimal256,
    pub total_bid_amount: Uint256,
    pub premium_rate: Decimal256,
    pub current_epoch: Uint128,
    pub current_scale: Uint128,
}
```

#### Example

```
{
  "bid_pools": [
    {
      "sum_snapshot": "1.0",
      "product_snapshot": "1.0",
      "total_bid_amount": "1000000",
      "premium_rate": "0.1",
      "current_epoch": "0",
      "current_scale": "0"
    }, 
    {
      "sum_snapshot": "1.0",
      "product_snapshot": "1.0",
      "total_bid_amount": "2000000",
      "premium_rate": "0.2",
      "current_epoch": "0",
      "current_scale": "0"
    }
  ]
}
```
