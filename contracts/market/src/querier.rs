use cosmwasm_bignumber::math::Decimal256;
use cosmwasm_std::{to_binary, Addr, Deps, QueryRequest, StdResult, WasmQuery};

use moneymarket::interest_model::{BorrowRateResponse, QueryMsg as InterestQueryMsg};
use moneymarket::overseer::{BorrowLimitResponse, QueryMsg as OverseerQueryMsg};

pub fn query_borrow_rate(
    deps: Deps,
    interest_addr: Addr,
    actual_peg: Decimal256,
) -> StdResult<BorrowRateResponse> {
    let borrow_rate: BorrowRateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: interest_addr.to_string(),
            msg: to_binary(&InterestQueryMsg::BorrowRate { actual_peg })?,
        }))?;

    Ok(borrow_rate)
}

pub fn query_borrow_limit(
    deps: Deps,
    overseer_addr: Addr,
    borrower: Addr,
    block_time: Option<u64>,
) -> StdResult<BorrowLimitResponse> {
    let borrow_limit: BorrowLimitResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: overseer_addr.to_string(),
            msg: to_binary(&OverseerQueryMsg::BorrowLimit {
                borrower: borrower.to_string(),
                block_time,
            })?,
        }))?;

    Ok(borrow_limit)
}
