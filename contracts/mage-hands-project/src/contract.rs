use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdError, StdResult, Storage, QueryResult, Coin, CosmosMsg, Uint128, BankMsg, HumanAddr,
};

use crate::msg::{HandleMsg, HandleAnswer, InitMsg, QueryMsg, QueryAnswer, ResponseStatus, ResponseStatus::Failure, ResponseStatus::Success};
use crate::state::{FUNDRAISING, EXPIRED, SUCCESSFUL, write_viewing_key, get_prng_seed, set_prng_seed, get_pledged_message, get_funded_message, get_funder, read_viewing_key, add_funds, get_title, get_description, get_deadline, get_goal, get_fee, set_fee, get_commission_addr, get_upfront, set_commission_addr, set_upfront, clear_funds, get_total, get_creator, get_status, set_creator, set_deadline, set_description, set_funded_message, set_goal, set_pledged_message, set_status, set_total, set_title,};
use primitive_types::U256;
use crate::u256_math::{div, mul, sub};
use crate::viewing_key::{VIEWING_KEY_SIZE, ViewingKey,};
use secret_toolkit::crypto::sha_256;

const DENOM: &str = "uscrt";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let prng_seed = sha_256(base64::encode(msg.entropy).as_bytes()).to_vec();
    set_prng_seed(&mut deps.storage, &prng_seed)?;

    let creator = deps.api.canonical_address(&msg.creator)?;
    set_creator(&mut deps.storage, &creator)?;

    let deadline = msg.deadline as u64;
    if env.block.time > deadline {
        return Err(StdError::generic_err("Cannot create project with deadline in the past"));
    }
    set_deadline(&mut deps.storage, deadline)?;
    set_title(&mut deps.storage, msg.title)?;
    set_description(&mut deps.storage, msg.description)?;
    let pledged_message = msg.pledged_message.unwrap_or_else(|| String::from(""));
    set_pledged_message(&mut deps.storage, pledged_message)?;
    let funded_message = msg.funded_message.unwrap_or_else(|| String::from(""));
    set_funded_message(&mut deps.storage, funded_message)?;
    
    let goal = msg.goal.u128();
    if goal == 0 {
        return Err(StdError::generic_err("Goal must be greater than 0"));
    }
    set_goal(&mut deps.storage, goal)?;

    let stored_fee = msg.fee.into_stored()?;
    set_fee(&mut deps.storage, stored_fee)?;
    set_upfront(&mut deps.storage, msg.upfront.u128())?;
    set_commission_addr(&mut deps.storage, &deps.api.canonical_address(&msg.commission_addr)?)?;

    set_status(&mut deps.storage, FUNDRAISING)?;
    set_total(&mut deps.storage, 0_u128)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::ChangeText { title, description, pledged_message, funded_message, .. } => try_change_text(deps, env, title, description, pledged_message, funded_message,),
        HandleMsg::Cancel { .. } => try_cancel(deps, env),
        HandleMsg::Contribute { anonymous, entropy, .. } => try_contribute(deps, env, anonymous, entropy),
        HandleMsg::Refund { .. } => try_refund(deps, env),
        HandleMsg::PayOut { .. } => try_pay_out(deps, env),
        HandleMsg::GenerateViewingKey { entropy, .. } => try_generate_viewing_key(deps, env, entropy),
    }
}

fn try_generate_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {
    let prng_seed = get_prng_seed(&deps.storage)?;

    let key = ViewingKey::new(&env, &prng_seed, (&entropy).as_ref());

    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    write_viewing_key(&mut deps.storage, &message_sender, &key);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::GenerateViewingKey { key })?),
    })
}

fn try_change_text<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    title: Option<String>,
    description: Option<String>,
    pledged_message: Option<String>,
    funded_message: Option<String>,
) -> StdResult<HandleResponse> {
    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
    let creator = get_creator(&deps.storage)?;
    if sender_address_raw != creator {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut updates: Vec<String> = vec![];

    if title.is_some() {
        set_title(&mut deps.storage, title.unwrap())?;
        updates.push(String::from("title"));
    }

    if description.is_some() {
        set_description(&mut deps.storage, description.unwrap())?;
        updates.push(String::from("description"));
    }

    if pledged_message.is_some() {
        set_pledged_message(&mut deps.storage, pledged_message.unwrap())?;
        updates.push(String::from("pledged message"));
    }

    if funded_message.is_some() {
        set_funded_message(&mut deps.storage, funded_message.unwrap())?;
        updates.push(String::from("funded message"));
    }

    let status;
    let msg;

    if updates.len() > 0 {
        status = Success;
        msg = format!("Updated {}", updates.join(", "));
    } else {
        status = Failure;
        msg = format!("Nothing was updated");
    }

    debug_print("text changed successfully");
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ChangeText { status, msg })?),
    })
}

fn try_contribute<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    anonymous: Option<bool>,
    entropy: String,
) -> StdResult<HandleResponse> {
    let status;
    let msg;
    let mut some_key: Option<ViewingKey> = None;

    let sent_coins = env.message.sent_funds.clone();
    if sent_coins[0].denom != DENOM {
        status = Failure;
        msg = String::from("Wrong denomination");
    } else {
        let amount = sent_coins[0].amount.u128();

        if amount == 0 {
            status = Failure;
            msg = String::from("No coins sent");
        } else {
            let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
            let anonymous = anonymous.unwrap_or(false);

            add_funds(&mut deps.storage, &sender_address_raw, anonymous, amount)?;

            let vk = read_viewing_key(&deps.storage, &sender_address_raw);
        
            if vk.is_none() {
                let key = ViewingKey::new(&env, &get_prng_seed(&deps.storage)?, (&entropy).as_ref());
                let message_sender = deps.api.canonical_address(&env.message.sender)?;
                write_viewing_key(&mut deps.storage, &message_sender, &key);

                some_key = Some(key);
            }

            status = Success;
            msg = format!("Successfully contributed {} uscrt", amount);
        }
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Contribute {
            status,
            msg,
            key: some_key,
        })?),
    })
}

fn try_cancel<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let mut response_status: ResponseStatus = Failure;
    let mut msg: String = String::from("");

    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
    let creator = get_creator(&deps.storage)?;
    if sender_address_raw != creator {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let status = get_status(&deps.storage)?;
    if status == EXPIRED {
        msg = String::from("Cannot cancel an expired project");
    } else if status == SUCCESSFUL {
        msg = String::from("Cannot cancel a funded project");
    } else if status == FUNDRAISING {
        response_status = Success;
        set_status(&mut deps.storage, EXPIRED)?;
    }

    debug_print("project cancelled successfully");
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Cancel { status: response_status, msg })?),
    })
}

pub fn try_refund<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let response_status;
    let msg;

    let mut messages = vec![];
    let status = get_status(&deps.storage)?;
    if status == SUCCESSFUL {
        response_status = Failure;
        msg = String::from("Cannot receive refund after project successfully funded");
    } else {
        let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
        let refund_amount = clear_funds(&mut deps.storage, &sender_address_raw)?;

        if refund_amount == 0 {
            response_status = Failure;
            msg = String::from("No funds to refund");
        } else {
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address.clone(),
                to_address: env.message.sender,
                amount: vec![Coin {
                    denom: DENOM.to_string(),
                    amount: Uint128(refund_amount),
                }],
            }));
            response_status = Success;
            msg = format!("{} uscrt refunded", refund_amount);
        }
    }

    debug_print("refund processed successfully");
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Refund {
            status: response_status,
            msg,
        })?),
    })
}

fn try_pay_out<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let response_status;
    let msg;

    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
    let creator = get_creator(&deps.storage)?;
    if sender_address_raw != creator {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut messages = vec![];
    let status = get_status(&deps.storage)?;
    if status == SUCCESSFUL {
        let total = get_total(&deps.storage)?;
        let fee = get_fee(&deps.storage)?;
        
        if fee.commission_rate_nom == 0 {
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address.clone(),
                to_address: env.message.sender,
                amount: vec![Coin {
                    denom: DENOM.to_string(),
                    amount: Uint128(total),
                }],
            }));
            msg = format!("Pay out {} uscrt", total);
        } else {
            let total_u256 = Some(U256::from(total));
            let commission_rate_nom = Some(U256::from(fee.commission_rate_nom));
            let commission_rate_denom =
                Some(U256::from(fee.commission_rate_denom));
            let commission_addr = get_commission_addr(&deps.storage)?;
            let commission_addr_human = deps.api.human_address(&commission_addr)?;
            let upfront = get_upfront(&deps.storage)?;
            let upfront_u256 = Some(U256::from(upfront));

            let commission_amount = div(mul(total_u256, commission_rate_nom), commission_rate_denom)
            .ok_or_else(|| {
                StdError::generic_err(format!(
                    "Cannot calculate total {} * commission_rate_nom {} / commission_rate_denom {}",
                    total_u256.unwrap(),
                    commission_rate_nom.unwrap(),
                    commission_rate_denom.unwrap(),
                ))
            })?;

            let commission_amount_u128 = commission_amount.low_u128();
            if commission_amount_u128 < upfront || commission_amount_u128 == 0 { // take no commission
                messages.push(CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: env.message.sender,
                    amount: vec![Coin {
                        denom: DENOM.to_string(),
                        amount: Uint128(total),
                    }],
                }));
                msg = format!("Pay out {} uscrt", total);
            } else { // subtract upfront fee from commission
                let commission_amount = sub(Some(commission_amount), upfront_u256).ok_or_else(|| {
                    StdError::generic_err(format!(
                        "Cannot calculate commission_amouut {} - upfront {}",
                        commission_amount,
                        upfront_u256.unwrap(),
                    ))
                })?;

                let payment_amount = sub(total_u256, Some(commission_amount)).ok_or_else(|| {
                    StdError::generic_err(format!(
                        "Cannot calculate total {} - adjusted commission_amount {}",
                        total_u256.unwrap(),
                        commission_amount,
                    ))
                })?;

                let creator_human_addr = deps.api.human_address(&creator)?;
                messages.push(CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: creator_human_addr,
                    amount: vec![Coin {
                        denom: DENOM.to_string(),
                        amount: Uint128(payment_amount.low_u128()),
                    }],
                }));

                messages.push(CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address,
                    to_address: commission_addr_human,
                    amount: vec![Coin {
                        denom: DENOM.to_string(),
                        amount: Uint128(commission_amount.low_u128()),
                    }],
                }));

                msg = format!("Pay out {} uscrt: payment {}, fee {}", total, payment_amount, commission_amount.low_u128());
            }
        }

        response_status = Success;
    } else {
        response_status = Failure;
        msg = String::from("Cannot receive pay out unless project successfully funded");
    }

    debug_print("refund processed successfully");
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::PayOut {
            status: response_status,
            msg,
        })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Status {} => query_status(deps),
        _ => authenticated_queries(deps, msg),
    }
}

fn authenticated_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    let (addresses, key) = msg.get_validation_params();

    for address in addresses {
        let canonical_addr = deps.api.canonical_address(address)?;

        let expected_key = read_viewing_key(&deps.storage, &canonical_addr);

        if expected_key.is_none() {
            // Checking the key will take significant time. We don't want to exit immediately if it isn't set
            // in a way which will allow to time the command and determine if a viewing key doesn't exist
            key.check_viewing_key(&[0u8; VIEWING_KEY_SIZE]);
        } else if key.check_viewing_key(expected_key.unwrap().as_slice()) {
            return match msg {
                // Base
                QueryMsg::StatusAuth { address, .. } => query_status_auth(&deps, &address),
                _ => panic!("This query type does not require authentication"),
            };
        }
    }

    Err(StdError::unauthorized())
}

fn query_status<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let status_string;

    let status = get_status(&deps.storage)?;
    if status == FUNDRAISING {
        status_string = String::from("fundraising");
    } else if status == EXPIRED {
        status_string = String::from("expired");
    } else if status == SUCCESSFUL {
        status_string = String::from("successful");
    } else {
        return Err(StdError::generic_err("Error getting status"));
    }

    let creator = get_creator(&deps.storage)?;
    let creator = deps.api.human_address(&creator)?;

    let goal = get_goal(&deps.storage)?;
    let goal = Uint128(goal);

    let total = get_total(&deps.storage)?;
    let total = Uint128(total);

    let deadline = get_deadline(&deps.storage)?;
    let deadline = deadline as i32;

    let title = get_title(&deps.storage);
    let description = get_description(&deps.storage);

    to_binary(&QueryAnswer::Status { creator, status: status_string, goal, total, deadline, title, description, })
}

fn query_status_auth<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, address: &HumanAddr) -> QueryResult {
    let status_string;

    let status = get_status(&deps.storage)?;
    if status == FUNDRAISING {
        status_string = String::from("fundraising");
    } else if status == EXPIRED {
        status_string = String::from("expired");
    } else if status == SUCCESSFUL {
        status_string = String::from("successful");
    } else {
        return Err(StdError::generic_err("Error getting status"));
    }

    let creator = get_creator(&deps.storage)?;
    let creator = deps.api.human_address(&creator)?;

    let goal = get_goal(&deps.storage)?;
    let goal = Uint128(goal);

    let total = get_total(&deps.storage)?;
    let total = Uint128(total);

    let deadline = get_deadline(&deps.storage)?;
    let deadline = deadline as i32;

    let title = get_title(&deps.storage);
    let description = get_description(&deps.storage);

    let sender_address_raw = deps.api.canonical_address(&address)?;
    let stored_funder = get_funder(&deps.storage, &sender_address_raw);

    let mut pledged_message: Option<String> = None;
    let mut funded_message: Option<String> = None;
    let mut contribution: Option<Uint128> = None;

    match stored_funder {
        Ok(stored_funder) => {
            if stored_funder.amount > 0 {
                if status != EXPIRED {
                    pledged_message = Some(get_pledged_message(&deps.storage));
                }
                if status == SUCCESSFUL {
                    funded_message = Some(get_funded_message(&deps.storage));
                }
            }
            contribution = Some(Uint128(stored_funder.amount));
        },
        Err(_) => {}
    };

    to_binary(&QueryAnswer::StatusAuth { creator, status: status_string, goal, total, deadline, title, description, pledged_message, funded_message, contribution})
}