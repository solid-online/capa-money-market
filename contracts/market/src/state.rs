use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::{Addr, Deps, Order, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, ReadonlyBucket, ReadonlySingleton, Singleton};

use moneymarket::market::BorrowerInfoResponse;

pub const KEY_CONFIG: &[u8] = b"config";
pub const KEY_STATE: &[u8] = b"state";

const PREFIX_LIABILITY: &[u8] = b"liability";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub contract_addr: Addr,
    pub owner_addr: Addr,
    pub stable_contract: Addr,
    pub overseer_contract: Addr,
    pub collector_contract: Addr,
    pub liquidation_contract: Addr,
    pub oracle_contract: Addr,
    pub base_borrow_fee: Decimal256,
    pub fee_increase_factor: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub total_liabilities: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct BorrowerInfo {
    pub loan_amount: Uint256,
    pub loan_amount_without_interest: Uint256,
}

pub fn store_config(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}

pub fn store_state(storage: &mut dyn Storage, data: &State) -> StdResult<()> {
    Singleton::new(storage, KEY_STATE).save(data)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    ReadonlySingleton::new(storage, KEY_STATE).load()
}

pub fn store_borrower_info(
    storage: &mut dyn Storage,
    borrower: &Addr,
    liability: &BorrowerInfo,
) -> StdResult<()> {
    bucket(storage, PREFIX_LIABILITY).save(borrower.as_bytes(), liability)
}

pub fn read_borrower_info(storage: &dyn Storage, borrower: &Addr) -> BorrowerInfo {
    match bucket_read(storage, PREFIX_LIABILITY).load(borrower.as_bytes()) {
        Ok(v) => v,
        _ => BorrowerInfo {
            loan_amount: Uint256::zero(),
            loan_amount_without_interest: Uint256::zero(),
        },
    }
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_borrower_infos(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<Vec<BorrowerInfoResponse>> {
    let liability_bucket: ReadonlyBucket<BorrowerInfo> =
        bucket_read(deps.storage, PREFIX_LIABILITY);

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after);

    liability_bucket
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|elem| {
            let (k, v) = elem?;
            let borrower = String::from_utf8(k)?;
            Ok(BorrowerInfoResponse {
                borrower,
                loan_amount: v.loan_amount,
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
