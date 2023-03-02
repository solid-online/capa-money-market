# Oracle

The Oracle contract acts as the price source for the CAPA Money Market.
Stablecoin-denominated prices of bAssets are periodically reported by
oracle feeders, and are made queriable by other smart contracts in the
Capapult ecosystem.

## InstantiateMSG

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String, // Address of contract owner that can feed in price values
    pub base_asset: String,  //Asset which fed-in-prices will be denomited in
}
```
#### Example:

```
{
  "owner": "terra1...", 
  "base_asset": "stable0000" 
}
```

## ExecuteMsg

### UpdateConfig

Updates the configuration of the contract. Can only be issued by the owner.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,  // Address of new owner
    }
}
```

#### Example

```
{
  "update_config": {
    "owner": "terra1..." 
  }
}

```

### RegisterFeeder

Registers a feeder to the specified asset token

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    RegisterFeeder {
        asset: String, // Asset to register feeder
        feeder: String, // Address of feeder to register
    }
}
```


#### Example

```
{
  "register_feeder": {
    "asset": "terra1...", // Stringified Cw20 contract address
    "feeder": "terra1..." 
  }
}
```

### FeedPrice

Feeds new price data. Can only be issued by the owner.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    FeedPrice {
        prices: Vec<(String, Decimal256)>, // Vector of assets and their prices
    }
}
```


#### Example

```
{
  "feed_price": {
    "prices": [
      ["terra1...", "123.456789"], // (Stringified Cw20 contract address, price)
      ["terra1...", "123.456789"] 
    ]
  }
}
```


## QueryMsg

### Config

Gets the Oracle contract configuration.

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
    pub owner: String, // Address of contract owner
    pub base_asset: String, // Asset in which fed-in prices will be denominated
}
```


#### Example

```
{
  "owner": "terra1...", 
  "base_asset": "stable0000" 
}
```

### Feeder

Gets the feeder for the specified asset.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Feeder {
        asset: String, // Asset to get feeder information
    }
}
```


#### Example

```
{
  "feeder": {
    "asset": "terra1..." // Stringified Cw20 Token contract address
  }
}
```

### FeederResponse


```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeederResponse {
    pub asset: String, // Asset type
    pub feeder: String, // Address of feeder allowed to feed prices for this asset
}
```


#### Example

```
{
  "asset": "terra1...", // Stringified Cw20 Token contract address
  "feeder": "terra1..." 
}
```

### Price

Gets price information for the specified base asset denominated in the quote asset.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Price {
        base: String, // Asset for which to get price
        quote: String, // Asset in which calcultaed price will be denominated
    }
}
```


#### Example

```
{
  "price": {
    "base": "terra1...", // Asset token contract HumanAddr in String form
    "quote": "stable0000" 
  }
}
```

### PriceResponse


```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PricesResponse {
    pub prices: Vec<PricesResponseElem>, // Vector of Asset price information
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PricesResponseElem {
    pub asset: String, // Asset whose price is being read
    pub price: Decimal256, // Price of Asset
    pub last_updated_time: u64, // Unix block timestamp when the price was last updated
}
```

#### Example

```
{
  "prices": [
    {
      "asset": "terra1...", // Stringified Cw20 token contract HumanAddr
      "price": "123.45678", 
      "last_updated_time": 10000 
    }
    {
      "asset": "terra1...", // Stringified Cw20 token contract HumanAddr
      "price": "123.45678", 
      "last_updated_time": 10000 
    }
  ]
}

```