use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::math::Uint256;
use cosmwasm_std::{Addr, Deps, Order, StdResult, Storage};
use cosmwasm_storage::{Bucket, ReadonlyBucket, ReadonlySingleton, Singleton};
use moneymarket::custody::BorrowerResponse;

const KEY_CONFIG: &[u8] = b"config";
const PREFIX_BORROWER: &[u8] = b"borrower";
const KEY_CONTRACT_INFO: &[u8] = b"contract_balance_info";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub collateral_token: Addr,
    pub overseer_contract: Addr,
    pub market_contract: Addr,
    pub liquidation_contract: Addr,
    pub collector_contract: Addr,
    pub max_deposit: Uint256,
}
// Total luna held by contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractBalanceInfo {
    pub balance: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct BorrowerInfo {
    pub balance: Uint256,
    pub spendable: Uint256,
}

pub fn store_config(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}

pub fn store_contract_balance_info(
    storage: &mut dyn Storage,
    data: &ContractBalanceInfo,
) -> StdResult<()> {
    Singleton::new(storage, KEY_CONTRACT_INFO).save(data)
}

pub fn read_contract_balance_info(storage: &dyn Storage) -> StdResult<ContractBalanceInfo> {
    ReadonlySingleton::new(storage, KEY_CONTRACT_INFO).load()
}

pub fn store_borrower_info(
    storage: &mut dyn Storage,
    borrower: &Addr,
    borrower_info: &BorrowerInfo,
) -> StdResult<()> {
    let mut borrower_bucket: Bucket<BorrowerInfo> = Bucket::new(storage, PREFIX_BORROWER);
    borrower_bucket.save(borrower.as_bytes(), borrower_info)?;

    Ok(())
}

pub fn remove_borrower_info(storage: &mut dyn Storage, borrower: &Addr) {
    let mut borrower_bucket: Bucket<BorrowerInfo> = Bucket::new(storage, PREFIX_BORROWER);
    borrower_bucket.remove(borrower.as_bytes());
}

pub fn read_borrower_info(storage: &dyn Storage, borrower: &Addr) -> BorrowerInfo {
    let borrower_bucket: ReadonlyBucket<BorrowerInfo> =
        ReadonlyBucket::new(storage, PREFIX_BORROWER);
    match borrower_bucket.load(borrower.as_bytes()) {
        Ok(v) => v,
        _ => BorrowerInfo {
            balance: Uint256::zero(),
            spendable: Uint256::zero(),
        },
    }
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_borrowers(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<Vec<BorrowerResponse>> {
    let position_bucket: ReadonlyBucket<BorrowerInfo> =
        ReadonlyBucket::new(deps.storage, PREFIX_BORROWER);

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after);

    position_bucket
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (k, v) = item?;
            let borrower: String = String::from_utf8(k)?;
            Ok(BorrowerResponse {
                borrower: deps.api.addr_validate(&borrower)?.to_string(),
                balance: v.balance,
                spendable: v.spendable,
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
