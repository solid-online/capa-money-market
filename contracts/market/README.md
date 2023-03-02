# Market

The Market contract acts as the point of interaction for all borrowing related activities. 
New stablecoin deposits are added to this contract's balance. 
Borrows are subtracted from this contract's balance.

## InstantiateMSG

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub owner_addr: String, // Owner address for config update
    pub stable_code_id: u64, // Solid token code ID used to instantiate
}
```
    
### Example:

```
{
  "owner_addr": "terra1...",
  "stable_code_id": "5"
}
```

## ExecuteMSG

### Receive

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg)
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Cw20ReceiveMsg {
    pub sender: String, // Sender of token transfer
    pub amount: Uint128, // Amount of tokens received 
    pub msg: Binary, // Base64-encoded string of JSON of Receive Hook
}
```

### Example:

```
{
  "receive": {
    "sender": "terra1...",
    "amount": "10000000",
    "msg": "eyAiZXhlY3V0ZV9tc2ciOiAiYmxhaCBibGFoIiB9"
  }
}
```

### RegisterContracts

Registers the addresses of other Money Market contracts. Can only be issued by the owner.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    RegisterContracts {
        overseer_contract: String, // The contract has the logics for Solid borrow interest rate
        interest_model: String, // Collector contract to send all the reserve
        collector_contract: String, // Faucet contract to drip Solid token to users
        liquidation_contract: String, // Liquidation contract address
        oracle_contract: String, // Oracle contract address
    }
}
```

### Example:

```
{
  "register_contracts": {
    "overseer_contract": "terra1...", 
    "interest_model": "terra1...", 
    "collector_contract": "terra1...", 
    "liquidation_contract": "terra1...", 
    "oracle_contract": "terra1..."
  }
}
```

### UpdateConfig 

Updates the configuration of the contract. Can be only issued by the owner

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  UpdateConfig {
          owner_addr: Option<String>, // Address of new owner
          interest_model: Option<String>, // New interest model contract address
          liquidation_contract: Option<String>, // New address of liquidation contract
  }
}
```

### Example:

```
{
  "update_config": {
    "owner_addr": "terra1...",  
    "interest_model": "terra1...", 
    "liquidation_contract": "terra1..."
  }
}
```

### ExecuteEpochOperations

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  ExecuteEpochOperations {},
}
```

### Example:

```
{
  "ExecuteEpochOperations": {},
}
```

### BorrowStable 

Borrow Solid with collaterals in overseer contract

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  BorrowStable {
          borrow_amount: Uint256, // Amount of stablecoins to borrow
          to: Option<String>, // Withdrawal address for borrowed stablecoins
  }
}
```

### Example:

```
{
  "borrow_stable": {
    "borrow_amount": "1000000000", 
    "to": "terra1..." 
  }
}
```

## Receive Hooks

### RepayStable

Repays previous stablecoin liability. Requires Solid to be sent with the message.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    RepayStable {}
}
```

### Example

```
{
  "repay_stable": {}
}
```

### RepayStableFromLiquidation

Repays a liquidated loan using a "Send CW20Msg" from Solid gained from liquidated collaterals. Can only be issued by liquidation_contract.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    RepayStableFromLiquidation {
        borrower: String // Borrower address
    }
}
```

### Example

```
{
  "repay_stable_from_liquidation": {
    "borrower": "terra1...",
  }
}
```

## QueryMSG

### Config

Gets the Market contract configuration.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {}
}
```

### Example:

```
{
  "config": {}
}
```

### ConfigResponse

Define a custom struct for each query response

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner_addr: String, // Address of contract owner
    pub stable_contract: String, // Address of stable contract
    pub interest_model: String, // Contract address of Interest Model
    pub overseer_contract: String, // Contract address of Overseer
    pub collector_contract: String, // Contract address of Collector
    pub liquidation_contract: String, // Address of liquidation contract
    pub oracle_contract: String, // Address of oracle contract
}
```

### Example:

```
{
  "owner_addr": "terra1...", 
  "stable_contract:": "terra1...", 
  "interest_model": "terra1...",  
  "overseer_contract": "terra1...", 
  "collector_contract": "terra1...", 
  "liquidation_contract": "terra1...", 
  "oracle_contract:": "terra1...", 
}
```

### State

Gets state information of Market. Returns an interest-accrued value if block_height field is filled. Returns the stored (no interest accrued) state if not filled.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    State {
        block_height: Option<u64>, // Block number to use in query
    }
}
```

### Example:

```
{
  "state": {
    "block_height": 123456, 
  }
}
```

### StateResponse

We define a custom struct for each query response

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_liabilities: Decimal256, // Total amount of liabilities of all borrowers
    pub last_interest_updated: u64, // Block number when interest was last accrued
    pub global_interest_index: Decimal256, // Current global interest index
}
```

### Example:

```
{
  "total_liabilities": "123.456789",
  "last_interest_updated": 123456789,
  "global_interest_index": "1.23456789",
}
```

### BorrowerInfo

Gets information for the specified borrower. Returns an interest-and-reward-accrued value if block_height field is filled. Returns the stored (no interest / reward accrued) state if not filled.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    BorrowerInfo {
        borrower: String, // Address of borrower
        block_height: Option<u64>, // Current block number
    }
}
```

### Example:

```
{
  "borrower_info": {
    "borrower": "terra1...", 
    "block_height": 123456 
  }
}
```

### BorrowInfoResponse

We define a custom struct for each query response

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerInfoResponse {
    pub borrower: String, // Address of borrower
    pub interest_index: Decimal256, // Interest index of borrower
    pub loan_amount: Uint256, // Amount of borrower's liability
}
```

### Example:

```
{
  "borrower": "terra1...", 
  "interest_index": "1.23456789",  
  "loan_amount": "123456789",
}
```

### BorrowerInfos

Gets information for all borrowers.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    BorrowInfos {
        start_after: Option<String>, // Borrower address to start query
        limit: Option<u32>,  // Maximum number of entries to query
    }
}
```

### Example:

```
{
  "borrower_infos": {
    "start_after": "terra1...", 
    "limit": 10 
  }
}
```

### BorrowerInfosResponse

We define a custom struct for each query response

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerInfosResponse {
    pub borrower_infos: Vec<BorrowerInfoResponse>, // List of borrower information
}
```

### Example:

```
{
  "borrower_infos": [
    {
      "borrower": "terra1...", 
      "interest_index": "1.23456789", 
      "loan_amount": "123456789", 
    }, 
    {
      "borrower": "terra1...", 
      "interest_index": "1.23456789", 
      "loan_amount": "123456789", 
    }  
  ]
}
```