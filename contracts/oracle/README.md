# Oracle

The Oracle contract acts as the price source for the CAPA Money Market.

When an asset is registered it is necessary to specify the source.

By source we mean the way in which the price is calculated.
Compared to the previous version (which allowed to manage prices only from feeder) is now possible to determine the source in the following ways:

- `Feeder`: When registered, a feeder address must be specified. The feeder can periodically update the prices of this type of asset, which are saved (like old version).

  
- `LsdContractQuery`: Currently Capapult support LSD tokens as collateral (LunaX, ampLuna, bLuna) and a offchain script compute the price of LSDs and feed them to the `oracle` contract. However, the rate of LSD/luna is present onchain and it can be queried to specific contracts.

  The price of LSD is : $base\_asset\_price *  LSD\_ratio$

  This allow to have the compute the price of LSDs if the `base_asset` (luna) is alredy registered and avaiable in the `oracle`. To do this, the following fields must be specified:
    - `base_asset`: The underlying asset of LSD (it must to be alredy registered);
    - `contract`: The contract to query the ratio;
    - `query_msg`: Binary msg to send;
    - `path_key`: `vec<String>` that specify the position of the ratio in the response;
    - `is_reverted`: In case the ratio is reversed, the division will be done instead of the multiplication;

- `AstroportLpVault`: One of the fundamental steps for integrating Capapult with the rest of the ecosystem is the use of LP tokens as collateral.
  
  Since provide the LP tokens directly involves the loss of staking rewards for users, the best solution is to accept **cTokens/vaultTokens** (the LP token recived for deposit the pool LP into a vault that handle the rewards).
  
  Currently **Any** vault that deposit LP on `AstroportGenerator` and give to the User a LP vault are supported (like `Spectrum` / `Eris` vault).
  To determinate the price of LPs, the prices of the **pool assets** must be recorded in the `oracle`.

  The ***adapted*** formula used can be found [here](https://blog.alphafinance.io/fair-lp-token-pricing/):

  $$total\_pool\_value= n*\sqrt[n]{\prod_{i=1}^n amount\_in\_pool_i * price_i}$$

  $$single\_vault\_lp\_value = \frac{total\_pool\_value * \frac{lp\_staked}{lp\_supply}}{vault\_lp\_supply}$$

  Where ***n*** is the number of the assets in the pool (***currently only pools with 2 assets are supported***).

  The following fields must be specified:
  - `vault_contract`: The contract of the vault;
  - `generator_contract`: Astroport generator contract;
  - `pool_contract`: The contract of the pool;
  
This type of structure allows to easily implement new methods to compute the price.

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

### RegisterAsset

Registers a new asset

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    RegisterAsset {
        asset: String, // Asset denom
        source: RegisterSource, // Type of soruce
    },
}
```


#### Example

```
{
  "register_feeder": {
    "asset": "terra1...", // Asset denom
    "source": {
      "feeder" : {
        "feeder": "terra1..." // Feeder address
      }
    }
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

### SourceInfo

Gets the source info for the specified asset.

```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    SourceInfo {
        asset: String, // Asset to get source information
    }
}
```


#### Example

```
{
  "source_info": {
    "asset": "terra1..." // Asset denom
  }
}
```

### SourceInfoResponse


```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SourceInfoResponse {
    pub source: String, // Soruce informations
}
```


#### Example

```
{
  "source": {
    "lsd_contract_query":{
        "base_asset": "uluna", // Base asset denom
        "contract": "terra1...", // Contract to query for ratio
        query_msg: "eyJjb25maWciOnt9fQ==", // Query msg in binary
        path_key: ["exchange_rate"], // Path of ratio in the response 
        is_inverted: false, // If true, a division will be performed instead of a multiplication
      }
    }
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