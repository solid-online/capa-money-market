use cosmwasm_bignumber::math::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, to_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg,
    WasmMsg,
};

use crate::error::ContractError;
use crate::querier::{query_borrower_info, query_liquidation_amount};
use crate::state::{
    read_all_collaterals, read_collaterals, read_config, read_whitelist_elem, store_collaterals,
    Config, WhitelistElem,
};

use moneymarket::custody::ExecuteMsg as CustodyExecuteMsg;
use moneymarket::liquidation_queue::LiquidationAmountResponse;
use moneymarket::market::BorrowerInfoResponse;
use moneymarket::oracle::PriceResponse;
use moneymarket::overseer::{AllCollateralsResponse, BorrowLimitResponse, CollateralsResponse};
use moneymarket::querier::{query_price, TimeConstraints};
use moneymarket::tokens::{Tokens, TokensHuman, TokensMath, TokensToHuman, TokensToRaw};

pub fn lock_collateral(
    deps: DepsMut,
    info: MessageInfo,
    collaterals_human: TokensHuman,
) -> Result<Response, ContractError> {
    let mut cur_collaterals: Tokens = read_collaterals(deps.storage, &info.sender);

    let collaterals: Tokens = collaterals_human.to_raw(deps.as_ref())?;

    cur_collaterals.add(collaterals.clone());
    store_collaterals(deps.storage, &info.sender, &cur_collaterals)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    for collateral in collaterals {
        let whitelist_elem: WhitelistElem = read_whitelist_elem(deps.storage, &collateral.0)?;
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: whitelist_elem.custody_contract.to_string(),
            funds: vec![],
            msg: to_binary(&CustodyExecuteMsg::LockCollateral {
                borrower: info.sender.to_string(),
                amount: collateral.1,
            })?,
        }));
    }

    // Logging stuff, so can be removed
    let collateral_logs: Vec<String> = collaterals_human
        .iter()
        .map(|c| format!("{}{}", c.1, c.0))
        .collect();

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "lock_collateral"),
        attr("borrower", info.sender),
        attr("collaterals", collateral_logs.join(",")),
    ]))
}

pub fn unlock_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collaterals_human: TokensHuman,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    let market = config.market_contract;

    let borrower = info.sender;
    let mut cur_collaterals: Tokens = read_collaterals(deps.storage, &borrower);
    let collaterals: Tokens = collaterals_human.to_raw(deps.as_ref())?;

    // Underflow check is done in sub_collateral
    if cur_collaterals.sub(collaterals.clone()).is_err() {
        return Err(ContractError::UnlockExceedsLocked {});
    }

    // Compute borrow limit with collaterals except unlock target collaterals
    let (borrow_limit, _) = compute_borrow_limit(
        deps.as_ref(),
        &cur_collaterals,
        Some(env.block.time.seconds()),
    )?;
    let borrow_amount_res: BorrowerInfoResponse =
        query_borrower_info(deps.as_ref(), market, borrower.clone())?;
    if borrow_limit < borrow_amount_res.loan_amount {
        return Err(ContractError::UnlockTooLarge(borrow_limit.into()));
    }

    store_collaterals(deps.storage, &borrower, &cur_collaterals)?;

    let mut messages: Vec<SubMsg> = vec![];
    for collateral in collaterals {
        let whitelist_elem: WhitelistElem = read_whitelist_elem(deps.storage, &collateral.0)?;
        messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: whitelist_elem.custody_contract.to_string(),
            funds: vec![],
            msg: to_binary(&CustodyExecuteMsg::UnlockCollateral {
                borrower: borrower.to_string(),
                amount: collateral.1,
            })?,
        })));
    }

    // Logging stuff, so can be removed
    let collateral_logs: Vec<String> = collaterals_human
        .iter()
        .map(|c| format!("{}{}", c.1, c.0))
        .collect();

    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(vec![
            attr("action", "unlock_collateral"),
            attr("borrower", borrower),
            attr("collaterals", collateral_logs.join(",")),
        ]))
}

pub fn liquidate_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    borrower: Addr,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    let market = config.market_contract;

    let borrower_validated = deps.api.addr_validate(borrower.as_str())?;
    let mut cur_collaterals: Tokens = read_collaterals(deps.storage, &borrower_validated);

    // Compute borrow limit with collaterals except unlock target collaterals
    let (borrow_limit, collateral_prices) = compute_borrow_limit(
        deps.as_ref(),
        &cur_collaterals,
        Some(env.block.time.seconds()),
    )?;
    let borrow_amount_res: BorrowerInfoResponse =
        query_borrower_info(deps.as_ref(), market, borrower_validated.clone())?;
    let borrow_amount = borrow_amount_res.loan_amount;

    // borrow limit is equal or bigger than loan amount
    // cannot liquidation collaterals
    if borrow_limit >= borrow_amount {
        return Err(ContractError::CannotLiquidateSafeLoan {});
    }

    let liquidation_amount_res: LiquidationAmountResponse = query_liquidation_amount(
        deps.as_ref(),
        config.liquidation_contract,
        borrow_amount,
        borrow_limit,
        &cur_collaterals.to_human(deps.as_ref())?,
        collateral_prices,
    )?;

    let liquidation_amount = liquidation_amount_res.collaterals.to_raw(deps.as_ref())?;

    // Store left collaterals
    cur_collaterals.sub(liquidation_amount.clone())?;
    store_collaterals(deps.storage, &borrower_validated, &cur_collaterals)?;

    let liquidation_messages: Vec<CosmosMsg> = liquidation_amount
        .iter()
        .filter(|amount| amount.1.ge(&Uint256::one()))
        .map(|collateral| {
            let whitelist_elem: WhitelistElem = read_whitelist_elem(deps.storage, &collateral.0)?;
            // OVERSEER
            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: whitelist_elem.custody_contract.to_string(),
                funds: vec![],
                msg: to_binary(&CustodyExecuteMsg::LiquidateCollateral {
                    liquidator: info.sender.to_string(),
                    borrower: borrower_validated.to_string(),
                    amount: collateral.1,
                })?,
            }))
        })
        .filter(|msg| msg.is_ok())
        .collect::<StdResult<Vec<CosmosMsg>>>()?;

    Ok(Response::new().add_messages(liquidation_messages))
}

pub fn query_collaterals(deps: Deps, borrower: Addr) -> StdResult<CollateralsResponse> {
    let collaterals: Tokens =
        read_collaterals(deps.storage, &deps.api.addr_validate(borrower.as_str())?);

    Ok(CollateralsResponse {
        borrower: borrower.to_string(),
        collaterals: collaterals
            .iter()
            .map(|c| Ok((c.0.to_string(), c.1)))
            .collect::<StdResult<Vec<(String, Uint256)>>>()?,
    })
}

pub fn query_all_collaterals(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<AllCollateralsResponse> {
    let start_after = if let Some(start_after) = start_after {
        Some(deps.api.addr_validate(start_after.as_str())?)
    } else {
        None
    };

    let all_collaterals: Vec<CollateralsResponse> = read_all_collaterals(deps, start_after, limit)?;

    Ok(AllCollateralsResponse { all_collaterals })
}

#[allow(clippy::ptr_arg)]
pub(crate) fn compute_borrow_limit(
    deps: Deps,
    collaterals: &Tokens,
    block_time: Option<u64>,
) -> StdResult<(Uint256, Vec<Decimal256>)> {
    let config: Config = read_config(deps.storage)?;

    let mut borrow_limit: Uint256 = Uint256::zero();
    let mut collateral_prices: Vec<Decimal256> = vec![];
    for collateral in collaterals.iter() {
        let collateral_token = collateral.0.clone();
        let collateral_amount = collateral.1;

        let price: PriceResponse = query_price(
            deps,
            config.oracle_contract.clone(),
            (deps.api.addr_validate(collateral_token.as_str())?).to_string(),
            "uusd".to_string(),
            block_time.map(|block_time| TimeConstraints {
                block_time,
                valid_timeframe: config.price_timeframe,
            }),
        )?;

        let elem: WhitelistElem = read_whitelist_elem(deps.storage, &collateral.0)?;
        let collateral_value = collateral_amount * price.rate;
        borrow_limit += collateral_value * elem.max_ltv;
        collateral_prices.push(price.rate);
    }

    // returns borrow_limit with collaterals value in stable denom
    Ok((borrow_limit, collateral_prices))
}

pub fn query_borrow_limit(
    deps: Deps,
    borrower: Addr,
    block_time: Option<u64>,
) -> StdResult<BorrowLimitResponse> {
    let collaterals = read_collaterals(deps.storage, &deps.api.addr_validate(borrower.as_str())?);

    // Compute borrow limit with collaterals
    let (borrow_limit, _) = compute_borrow_limit(deps, &collaterals, block_time)?;

    Ok(BorrowLimitResponse {
        borrower: borrower.to_string(),
        borrow_limit,
    })
}
