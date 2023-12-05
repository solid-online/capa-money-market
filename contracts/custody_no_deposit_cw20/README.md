# Custody LunaX

The Custody contract is where supplied bAsset collaterals are managed. Users can make collateral
deposits and withdrawals to and from this contract. The Custody contract is also responsible for
claiming bAsset rewards and converting them to Terra stable coins, which are then sent to the [Overseer contract](../overseer) for eventual distribution.

## InstantiateMSG

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub owner: String, // Owner address
    pub collateral_token: String, // bAsset token address
    pub overseer_contract: String, // Overseer contract address
    pub market_contract: String, // Market contract address
    pub staking_contract: String, // Liquid staking staking contract
    pub liquidation_contract: String, // Liquidation contract address
    pub collector_contract: String, // Collector contract address
    pub stable_contract: String, // Contract address for Solid CW20
    pub astroport_pair: String, // Astroport pair contract address
    pub max_slipage: Decimal,
}
```
#### Example:

```
{
  "owner": "terra1...", 
  "collatera_token": "terra1...",
  "overseer_contract": "terra1...",
  "market_contract": "terra1...",
  "staking_contract": "terra1...",
  "liquidation_contract": "terra1...",
  "collector_contract": "terra1...",
  "stable_contract": "terra1...",
  "astroport_pair": "terra1...",
  "max_slipage": "2.0"
}
```

## ExecuteMsg

### Receive

Allows the token transfer to execute a deposit action within the same transaction.

```
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Cw20ReceiveMsg {
    pub sender: String, // Sender address
    pub amount: Uint128, // Amount of token sent
    pub msg: Binary, // Base64-encoded string of the deposit action
}
```

### Example
```
{
  "receive": {
    "sender": "terra1...",
    "amount": "100",
    "msg": "eyAiZXhlY3V0ZV9tc2ciOiAiYmluYXJ5IiB9"
}
```

### UpdateConfig

Updates the configuration of the contract. Can only be issued by the owner.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>, // Address of new owner
        liquidation_contract: Option<String>, // New contract address of Liquidation Contract
        staking_contract: Option<String>, // New contract address of Staking Contract
        collector_contract: Option<String>, // New contract address of Collector Contract
        max_slipage: Option<Decimal>,
    }
}
```

### Example

```
{
  "update_config": {
    "owner": "terra1...",
    "liquidation_contract": "terra1...",
    "staking_contract": "terra1...",
    "collector_contract": "terra1...",
    "max_slipage": "2.0"
   }
}
```

### LockCollateral

Locks borrower's collateral to be used in their loan position, decreasing the amount of spendable collateral. Can only be issued by Overseer.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    LockCollateral {
        borrower: String, // Borrower address 
        amount: Uint256, // Amount to lock
    }
}
```

### Example

```
{
  "lock_collateral": { 
    "borrower": "terra1...", 
    "amount": "100000"
  }
}
```

### UnlockCollateral

Unlocks borrower's collateral from their loan position, increasing the amount of spendable collateral. Can only be issued by Overseer.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UnlockCollateral {
        borrower: String, // Borrower address
        amount: Uint256, // Amount to unlock
    }
}
```

### Example

```
{
  "unlock_collateral": { 
    "borrower": "terra1...", 
    "amount": "100000"
  }
}
```

### DistrubuteRewards

Withdraws accrued rewards from the LunaX Contract, swaps rewards to Solid. Can only be issued by Overseer.
Afterwards, sends swapped rewards to Collector.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    DistributeRewards {}
}
```

### Example

```
{
  "distribute_rewards": {} 
}
```

### LiquidateCollateral

Liquidates specified amount of locked collateral. Can only be issued by Overseer.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    LiquidateCollateral {
        liquidator: String, // Liquidator address
        borrower: String, // Borrower address
        amount: Uint256, // Liquidatable amount
    }
}
```

### Example

```
{
  "liquidate_collateral": {
    "liquidator": "terra1...",
    "borrower": "terra1...",
    "amount": "100000"
  }
}
```

### WithdrawCollateral

Withdraws amount of spendable collateral. Withdraws all spendable collateral if the amount field is left empty.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    WithdrawCollateral { 
      amount: Option<Uint256>, // Amount to withdraw
      }
}
```

### Example

```
{
  "withdraw_collateral": { 
    "amount": "10000"
  }
}
```
## ReceiveHooks

### DepositCollateral

Deposited collaterals have to be locked in the Overseer before they can be utilized in a loan position.

Deposit collateral token.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    DepositCollateral {},
}
```

### Example

```
{
  "deposit_collateral": {}
}
```

## QueryMsg

### Config

Gets the LunaX custody contract configuration.

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
    pub owner: String, // Owner address
    pub collateral_token: String, // LunaX token contract address
    pub overseer_contract: String, // Overseer contract address
    pub market_contract: String, // Market contract address
    pub staking_contract: String, // Stacking contract address
    pub liquidation_contract: String, // Liquidation contract address
    pub stable_contract: String, // Stable contract address
    pub collector_contract: String, // Collector contract address
    pub max_slipage: String 
}
```

### Example

```
{
  "owner": "terra1...",
  "collateral_token": "terra1...",
  "overseer_contract": "terra1...",
  "market_contract": "terra1...",
  "staking_contract": "terra1...",
  "liquidation_contract": "terra1...",
  "stable_contract": "terra1...",
  "collector_contract": "terra1...",
  "max_slipage": "2.0"
}

```

### Borrower

Gets the collateral balance of the borrower.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Borrower {
        address: String, // Borrower address
    }
}
```

### Example

```
{
  "borrower": "terra1...",
}
```

### BorrowerResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerResponse {
    pub borrower: String, // Borrower address
    pub balance: Uint256, // Total amount deposited
    pub spendable: Uint256, // Spendable amount
}
```

### Example

```
{
  "borrower_response": {
    "borrower": "terra1...",
    "balance": "100000",
    "spendable": "10000"
}
```

### Borrowers

Get the balance of all borrowers.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Borrowers {
        start_after: Option<String>, // Starting index of borrowers query
        limit: Option<u32>, // Maximum number of borrowers
    }
}
```

### Example

```
{
  "borrowers": { 
    "start_after": "terra1...", 
    "limit": "15"
  }
}
```

### BorrowersResponse

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowersResponse {
    pub borrowers: Vec<BorrowerResponse>, // Vector for multiple borrowers
}
```

### Example

```
{
  "borrowers": [
    {
      "borrower": "terra1...", 
      "balance": "2389476982", 
      "spendable": "2837492" 
    }, 
    {
      "borrower": "terra1...", 
      "balance": "2389476982", 
      "spendable": "2837492" 
    }
  ]
}
```