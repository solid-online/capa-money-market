use cosmwasm_bignumber::math::Decimal256;
use cosmwasm_std::{
    attr, Addr, Attribute, DepsMut, Empty, Env, MessageInfo, QueryRequest, Response,
};
use moneymarket::oracle::{PathKey, RegisterSource, Source, UpdateSource};

use crate::{
    error::ContractError,
    functions::{get_price, pool_infos},
    state::{Config, ASSETS, CONFIG},
};

pub fn run_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}

pub fn run_register_asset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: String,
    source: RegisterSource,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if ASSETS.may_load(deps.storage, asset.clone())?.is_some() {
        return Err(ContractError::AssetAlreadyWhitelisted {});
    };

    let mut attributes: Vec<Attribute> = vec![];

    match source {
        RegisterSource::Feeder { feeder } => {
            ASSETS.save(
                deps.storage,
                asset.clone(),
                &Source::Feeder {
                    feeder,
                    price: None,
                    last_updated_time: None,
                },
            )?;

            attributes.push(attr("source_type", "feeder"));
        }
        RegisterSource::OnChainRate {
            base_asset,
            query,
            path_key,
            is_inverted,
        } => {
            register_on_chain_rate(
                deps,
                env,
                asset.clone(),
                base_asset,
                query,
                path_key,
                is_inverted,
            )?;

            attributes.push(attr("source_type", "lsd_contract_query"));
        }
        RegisterSource::AstroportLpVault {
            vault_contract,
            generator_contract,
            pool_contract,
        } => {
            register_clp_astro(
                deps,
                env,
                asset.clone(),
                vault_contract,
                generator_contract,
                pool_contract,
            )?;

            attributes.push(attr("source_type", "astroport_lp_autocompunt"));
        }
    }

    attributes.push(attr("asset", asset));

    Ok(Response::new().add_attributes(attributes))
}

pub fn run_update_source(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: String,
    update_source: UpdateSource,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut attributes: Vec<Attribute> = vec![];

    // In case asset is not registered this return an error
    match ASSETS.load(deps.storage, asset.clone()) {
        Ok(source) => match (source, update_source) {
            (
                Source::Feeder {
                    price,
                    last_updated_time,
                    ..
                },
                UpdateSource::Feeder { feeder },
            ) => {
                ASSETS.save(
                    deps.storage,
                    asset.clone(),
                    &Source::Feeder {
                        feeder,
                        price,
                        last_updated_time,
                    },
                )?;

                attributes.push(attr("source_type", "feeder"));
            }

            (
                Source::OnChainQuery {
                    base_asset,
                    query,
                    path_key,
                    is_inverted,
                },
                UpdateSource::OnChainRate {
                    base_asset: new_base_asset,
                    query: new_query,
                    path_key: new_path_key,
                    is_inverted: new_is_inverted,
                },
            ) => {
                let new_base_asset = if let Some(new_base_asset) = new_base_asset {
                    new_base_asset.into_option()
                } else {
                    base_asset
                };

                register_on_chain_rate(
                    deps,
                    env,
                    asset.clone(),
                    new_base_asset,
                    new_query.unwrap_or(query),
                    new_path_key.unwrap_or(path_key),
                    new_is_inverted.unwrap_or(is_inverted),
                )?;

                attributes.push(attr("source_type", "lsd_contract_query"));
            }

            (
                Source::AstroportLpVault {
                    vault_contract,
                    generator_contract,
                    pool_contract,
                    ..
                },
                UpdateSource::AstroportLpVault {
                    vault_contract: new_vault_contract,
                    generator_contract: new_generator_contract,
                    pool_contract: new_pool_contract,
                },
            ) => {
                register_clp_astro(
                    deps,
                    env,
                    asset.clone(),
                    new_vault_contract.unwrap_or(vault_contract),
                    new_generator_contract.unwrap_or(generator_contract),
                    new_pool_contract.unwrap_or(pool_contract),
                )?;

                attributes.push(attr("source_type", "astroport_lp_autocompunt"));
            }

            _ => return Err(ContractError::SourceIsNotFeeder { asset }),
        },
        Err(_) => return Err(ContractError::AssetIsNotWhitelisted { asset }),
    }

    attributes.push(attr("asset", asset));

    Ok(Response::new().add_attributes(attributes))
}

pub fn run_feed_prices(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    prices: Vec<(String, Decimal256)>,
) -> Result<Response, ContractError> {
    let mut attributes = vec![attr("action", "feed_prices")];
    for price in prices {
        let asset: String = price.0;
        let price: Decimal256 = price.1;

        match ASSETS.load(deps.storage, asset.clone()) {
            Ok(source) => match source {
                Source::Feeder { feeder, .. } => {
                    if feeder != info.sender {
                        return Err(ContractError::Unauthorized {});
                    }
                    if price.is_zero() {
                        return Err(ContractError::NotValidZeroPrice {});
                    }

                    attributes.push(attr("asset", asset.to_string()));
                    attributes.push(attr("price", price.to_string()));

                    ASSETS.save(
                        deps.storage,
                        asset,
                        &Source::Feeder {
                            feeder,
                            price: Some(price),
                            last_updated_time: Some(env.block.time.seconds()),
                        },
                    )?
                }
                #[allow(unreachable_patterns)]
                _ => return Err(ContractError::SourceIsNotFeeder { asset }),
            },
            Err(_) => return Err(ContractError::AssetIsNotWhitelisted { asset }),
        }
    }

    Ok(Response::new().add_attributes(attributes))
}

/// Properly register/update a LSD
#[allow(clippy::too_many_arguments)]
fn register_on_chain_rate(
    deps: DepsMut,
    env: Env,
    asset: String,
    base_asset: Option<String>,
    query: QueryRequest<Empty>,
    path_key: Vec<PathKey>,
    is_inverted: bool,
) -> Result<(), ContractError> {
    deps.api.addr_validate(&asset)?;

    ASSETS.save(
        deps.storage,
        asset.clone(),
        &Source::OnChainQuery {
            base_asset,
            query,

            path_key,
            is_inverted,
        },
    )?;

    // get_price is called to check if the passed data are valids, otheriwse the tx is reverted
    get_price(deps.as_ref(), env, asset)?;

    Ok(())
}

/// Properly register/update a clp_astro
fn register_clp_astro(
    deps: DepsMut,
    env: Env,
    asset: String,
    vault_contract: Addr,
    generator_contract: Addr,
    pool_contract: Addr,
) -> Result<(), ContractError> {
    let (assets, lp_contract) = pool_infos(deps.as_ref(), pool_contract.clone())?;

    ASSETS.save(
        deps.storage,
        asset.clone(),
        &Source::AstroportLpVault {
            vault_contract,
            generator_contract,
            pool_contract,
            lp_contract,
            assets,
        },
    )?;

    // get_price is called to check if the passed data are valids, otheriwse the tx is reverted
    get_price(deps.as_ref(), env, asset)?;

    Ok(())
}
