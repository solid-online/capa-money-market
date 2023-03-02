# Overseer

The Overseer contract is responsible for storing key protocol parameters
and the whitelisting of new bAsset collaterals. The Overseer keeps track of locked collateral amounts and calculates the borrow limits for each user.

This contract is the recipient for collected bAsset rewards claimed by
Custody contracts. The Overseer calculates the amount of depositor
subsidies to be distributed, and the resulting amount is sent to
the Market contract.

The Overseer halts borrow-related operations if the Oracle's price data is
older than 60 seconds `price_timeframe`. Operations are resumed when new
price data is fed-in.

## InstantiateMsg

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub owner_addr: String, // Owner address
    pub oracle_contract: String, // Oracle contract address
    pub market_contract: String, // Market contract address
    pub liquidation_contract: String, // Liquidation model contract address
    pub collector_contract: String, // Collector contract address 
    pub stable_contract: String, // Stable contract address
    pub epoch_period: u64, // # of blocks per epoch period
    pub price_timeframe: u64, // Window of time before price data is considered outdated (in seconds)
}
```

### Example

```
{
  "owner_addr": "terra1...", 
  "oracle_contract": "terra1...", 
  "market_contract": "terra1...", 
  "liquidation_contract": "terra1...", 
  "collector_contract": "terra1...", 
  "stable_contract": "terra1...",
  "epoch_period": "86400", 
  "price_timeframe": "60"
}
```

## ExecuteMsg

### UpdateConfig

Updates the configuration of the contract. Can only be issued by the owner.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    UpdateConfig {
        owner_addr: Option<String>, // New owner address
        oracle_contract: Option<String>, // New oracle contract address
        liquidation_contract: Option<String>, // New liquidation contract address
        epoch_period: Option<u64>, // # of blocks per epoch period
        price_timeframe: Option<u64>, // Window of time before price data is considered outdated (in seconds)
    },
}
```

### Example

```
{
  "update_config": { 
    "owner_addr": "terra1...", 
    "oracle_contract": "terra1...", 
    "liquidation_contract": "terra1...",
    "epoch_period": "86400", 
    "price_timeframe": "60",
  }
}
```

### Whitelist

Create new custody contract for the given collateral token. Can only be issued by the owner.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    Whitelist {
        name: String, // bAsset name
        symbol: String, // bAsset symbol
        collateral_token: String, // bAsset token contract
        custody_contract: String, // bAsset custody contract
        max_ltv: Decimal256, // Maximum loan To Value ratio
    },
}
```

### Example

```
{
  "whitelist": { 
    "name": "LunaX", 
    "symbol": "lunax", 
    "collateral_token": "terra1...", 
    "custody_contract": "terra1...", 
    "max_ltv": "0.75" 
  }
}
```

### UpdateWhitelist

Update registered whitelist info. Can only be issued by the owner.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
UpdateWhitelist {
        collateral_token: String, // Liquid asset token contract
        custody_contract: Option<String>, // Liquid asset custody contract
        max_ltv: Option<Decimal256>, // Loan To Value ratio
    },
}
```

### Example

```
{
  "update_whitelist": {
    "collateral_token": "terra1...", 
    "custody_contract": "terra1...", 
    "max_ltv": "0.75" 
  }
}
```

### ExecuteEpochOperations

Claims all staking rewards from the bAsset contracts and also do a epoch basis updates:
1. Invoke [Custody] DistributeRewards
2. Update epoch state

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    ExecuteEpochOperations {}
}
```

### Example

```
{
  "execute_epoch_operations": {}
}
```

### LockCollateral

Locks specified amount of collateral deposited.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    LockCollateral {
        collaterals: TokensHuman, // List of collaterals and their lock amounts
    }
}


pub type TokensHuman = Vec<(String, Uint256)>; // Vector of (Collateral token, Amount to lock)
```

### Example

```
{
  "lock_collateral": { 
    "collaterals": [
      ["terra1...", "100000000"], // <(Collateral Token, Amount)>
    ]
  }
}
```

### UnlockCollateral

Unlocks specified amount of collateral unlocked.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    UnlockCollateral {
        collaterals: TokensHuman, // List of collaterals and their lock amounts
    }
}

pub type TokensHuman = Vec<(String, Uint256)>; // Vector of (Collateral token, Amount to lock)
```

### Example

```
{
  "unlock_collateral": {
    "collaterals": [
      ["terra1...", "100000000"], // <(Collateral Token, Amount)>
    ]
  }
}
```

### LiquidateCollateral

Liquidates loan position of the specified borrower.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    LiquidateCollateral {
        borrower: String, // Borrower address
    }
}
```

### Example

```
{
  "liquidate_collateral": {
    "borrower": "terra1..."
  }
}
```

## QueryMsg

### Config

Gets Overseer contract configuration.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {}
}
```

### Example

```
{
  "config": {}
}
```

### ConfigResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner_addr: String, // Owner address
    pub oracle_contract: String, // Oracle contract address
    pub market_contract: String, // Market contract address
    pub liquidation_contract: String, // Liquidation contract address 
    pub collector_contract: String, // Collector contract address
    pub stable_contract: String, // Stable contract address
    pub epoch_period: u64, // # of blocks per epoch period
    pub price_timeframe: u64, // Window of time before price data is considered outdated (in seconds)
}
```

### Example

```
{
  "owner_addr": "terra1...", 
  "oracle_contract": "terra1...", 
  "market_contract": "terra1...", 
  "liquidation_contract": "terra1...", 
  "collector_contract": "terra1...",
  "stable_contract": "terra1...",
  "epoch_period": "86400", 
  "price_timeframe": "60" 
}
```

### EpochState

Gets information of the current epoch.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    EpochState {}
}
```

### Example

```
{
  "epoch_state": {}
}
```

### EpochStateResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EpochState {
    pub deposit_rate: Decimal256,
    pub prev_stable_supply: Uint256,
    pub prev_exchange_rate: Decimal256,
    pub prev_interest_buffer: Uint256,
    pub last_executed_height: u64, // Block number when epoch operations were last executed
}
```

### Example

```
{
  "deposit_rate": "0.13", 
  "prev_stable_supply": "1000000", 
  "prev_exchange_rate": "1.2", 
  "prev_interest_buffer": "1000000", 
  "last_executed_height": "123456" 
}
```

### Whitelist

Gets information about the specified collateral if the collateral_token field is filled. 
Gets information about all collaterals if the collateral_token field is not filled.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Whitelist {
        collateral_token: Option<String>, // Cw20 Token address of collateral to query information
        start_after: Option<String>, // Collateral Cw20 Token address to start query
        limit: Option<u32>, // Maximum number of query entries
    }
}
```

### Example

```
{
  "whitelist": { 
    "collateral_token": null, 
    "start_after": "terra1...", 
    "limit": "3" 
  }
}
```

### WhitelistResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistResponse {
    pub elems: Vec<WhitelistResponseElem>, // Vector of whitelisted collateral information
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistResponseElem {
    pub name: String, // Name of liquid asset collateral
    pub symbol: String, // Token symbol of liquid asset collateral
    pub max_ltv: Decimal256, // Loan-to-value ratio allowed for collateral
    pub custody_contract: String, // Custody contract address of this collateral
    pub collateral_token: String, // Cw20 Token contract address of this collateral
}
```

### Example

```
{
  "elems": [
    {
      "name": "LunaX", 
      "symbol": "lunax", 
      "max_ltv": "0.5", 
      "custody_contract": "terra1...", 
      "collateral_token": "terra1..." 
    }, 
    {
      "name": "bonded atom", 
      "symbol": "ubatom", 
      "max_ltv": "0.4", 
      "custody_contract": "terra1...", 
      "collateral_token": "terra1..." 
    }
  ]
}
```

### Collaterals

Gets locked collateral information for the specified borrower.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Collaterals {
        borrower: String, // Borrower address
    }
}
```

### Example

```
{
  "collaterals": {
    "borrower": "terra1..." 
  }
{
```

### CollateralsResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollateralsResponse {
    pub borrower: String, // Borrower address
    pub collaterals: TokensHuman, // List of collaterals and locked amounts
}

pub type TokensHuman = Vec<(String, Uint256)>; // <(Collateral Token, Amount)>
```

### Example

```
{
  "borrower": "terra1...", 
  "collaterals": [
    ["terra1...", "100000000"],  // <(Collateral Token, Amount)>
  ] 
}
```

### AllCollaterals

Gets locked collateral information for all borrowers.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    AllCollaterals {
        start_after: Option<String>, // Borrower address of start query
        limit: Option<u32>, // Maximum number of query entries
    }
}
```

### Example 

```
{
  "all_collaterals": {
    "start_after": "terra1...", 
    "limit": 8 
  }
}
```

### AllCollateralsResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllCollateralsResponse {
    pub all_collaterals: Vec<CollateralsResponse>, // List of collaterals and locked amounts
}
```

### Example

```
{
  "all_collaterals": [
    {
      "borrower": "terra1...", 
      "collaterals": [
        ["terra1...", "100000000"], // <(Collateral Token, Amount)>
      ]
    }, 
    {
      "borrower": "terra1...", 
      "collaterals": [
        ["terra1...", "100000000"], // <(Collateral Token, Amount)>
      ]
    }
  ]
}
```

### BorrowLimit

Gets the borrow limit for the specified borrower.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    BorrowLimit {
        borrower: String, // Borrow address
        block_time: Option<u64>, // Current block timestamp
    }
}
```

### Example

```
{
  "borrow_limit": { 
    "borrower": "terra1...", 
    "block_time": "123456" 
  }
}
```

### BorrowLimitResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowLimitResponse {
    pub borrower: String, // Borrow address
    pub borrow_limit: Uint128, // Borrow limit
}
```

### Example

```
{
  "borrower": "terra1...", 
  "borrow_limit": "10000000",
}
```