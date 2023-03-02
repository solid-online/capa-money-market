use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::{Addr, Deps, Order, StdError, StdResult, Storage};
use cosmwasm_storage::{Bucket, ReadonlyBucket, ReadonlySingleton, Singleton};

use moneymarket::overseer::{CollateralsResponse, WhitelistResponseElem};
use moneymarket::tokens::Tokens;

const KEY_CONFIG: &[u8] = b"config";

const PREFIX_WHITELIST: &[u8] = b"whitelist";
const PREFIX_COLLATERALS: &[u8] = b"collateral";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub owner_addr: Addr,
    pub oracle_contract: Addr,
    pub market_contract: Addr,
    pub liquidation_contract: Addr,
    pub collector_contract: Addr,
    pub stable_contract: Addr, // pub stable_denom: String,
    pub price_timeframe: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct WhitelistElem {
    pub name: String,
    pub symbol: String,
    pub max_ltv: Decimal256,
    pub custody_contract: Addr,
}

pub fn store_config(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}

pub fn store_whitelist_elem(
    storage: &mut dyn Storage,
    collateral_token: &Addr,
    whitelist_elem: &WhitelistElem,
) -> StdResult<()> {
    let mut whitelist_bucket: Bucket<WhitelistElem> = Bucket::new(storage, PREFIX_WHITELIST);
    whitelist_bucket.save(collateral_token.as_bytes(), whitelist_elem)?;

    Ok(())
}

pub fn read_whitelist_elem(
    storage: &dyn Storage,
    collateral_token: &Addr,
) -> StdResult<WhitelistElem> {
    let whitelist_bucket: ReadonlyBucket<WhitelistElem> =
        ReadonlyBucket::new(storage, PREFIX_WHITELIST);
    match whitelist_bucket.load(collateral_token.as_bytes()) {
        Ok(v) => Ok(v),
        _ => Err(StdError::generic_err(
            "Token is not registered as collateral",
        )),
    }
}

pub fn read_whitelist(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<Vec<WhitelistResponseElem>> {
    let whitelist_bucket: ReadonlyBucket<WhitelistElem> =
        ReadonlyBucket::new(deps.storage, PREFIX_WHITELIST);

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after);

    whitelist_bucket
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|elem| {
            let (k, v) = elem?;
            let collateral_token = String::from_utf8(k)?;
            let custody_contract = v.custody_contract.to_string();
            Ok(WhitelistResponseElem {
                name: v.name,
                symbol: v.symbol,
                collateral_token,
                custody_contract,
                max_ltv: v.max_ltv,
            })
        })
        .collect()
}

#[allow(clippy::ptr_arg)]
pub fn store_collaterals(
    storage: &mut dyn Storage,
    borrower: &Addr,
    collaterals: &Tokens,
) -> StdResult<()> {
    let mut collaterals_bucket: Bucket<Tokens> = Bucket::new(storage, PREFIX_COLLATERALS);
    if collaterals.is_empty() {
        collaterals_bucket.remove(borrower.as_bytes());
    } else {
        collaterals_bucket.save(borrower.as_bytes(), collaterals)?;
    }

    Ok(())
}

pub fn read_collaterals(storage: &dyn Storage, borrower: &Addr) -> Tokens {
    let collaterals_bucket: ReadonlyBucket<Tokens> =
        ReadonlyBucket::new(storage, PREFIX_COLLATERALS);
    match collaterals_bucket.load(borrower.as_bytes()) {
        Ok(v) => v,
        _ => vec![],
    }
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_all_collaterals(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<Vec<CollateralsResponse>> {
    let whitelist_bucket: ReadonlyBucket<Tokens> =
        ReadonlyBucket::new(deps.storage, PREFIX_COLLATERALS);

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after);

    whitelist_bucket
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|elem| {
            let (k, v) = elem?;
            let borrower = String::from_utf8(k)?;
            let collaterals: Vec<(String, Uint256)> = v
                .iter()
                .map(|c| Ok((c.0.to_string(), c.1)))
                .collect::<StdResult<Vec<(String, Uint256)>>>()?;

            Ok(CollateralsResponse {
                borrower,
                collaterals,
            })
        })
        .collect()
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<Addr>) -> Option<Vec<u8>> {
    start_after.map(|addr| {
        let mut v = addr.as_bytes().to_vec();
        v.push(1);
        v
    })
}
