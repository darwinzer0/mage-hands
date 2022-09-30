use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Env, Addr,
    Response, StdError, StdResult, Uint128, DepsMut, Deps, MessageInfo,
};

use crate::msg::{
    ExecuteAnswer, ExecuteMsg, InstantiateMsg, PlatformExecuteMsg, QueryAnswer, QueryMsg, ResponseStatus,
    ResponseStatus::Failure, ResponseStatus::Success, PlatformQueryMsg, ValidatePermitResponse,
    ExecuteReceiveMsg,
};
use crate::reward::{RewardMessage};
use crate::state::{
    get_subtitle, set_subtitle,
    add_funds, clear_funds, get_categories, get_creator, get_deadline,
    get_description, get_funded_message, get_funder, get_goal, get_pledged_message,
    get_prng_seed, get_status, get_title, get_total, is_paid_out, paid_out,
    read_viewing_key, set_categories, set_creator, set_deadline,
    set_description, set_funded_message, set_goal, set_pledged_message, set_prng_seed,
    set_status, set_title, set_total, write_viewing_key, EXPIRED, FUNDRAISING,
    SUCCESSFUL, set_config, get_config, set_deadman, get_deadman,
    push_comment, get_comments, set_spam_flag, get_spam_count, set_snip24_reward, set_reward_messages, get_reward_messages,
};
use crate::utils::space_pad;
use crate::viewing_key::{ViewingKey, VIEWING_KEY_SIZE};
use secret_toolkit::{
    crypto::sha_256,
    permit::{Permit,},
    snip20::{
        register_receive_msg, transfer_msg, set_viewing_key_msg,
    },
    utils::{HandleCallback, Query},
};

pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";
pub const RESPONSE_BLOCK_SIZE: usize = 256;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let prng_seed = sha_256(base64::encode(msg.entropy).as_bytes()).to_vec();
    set_prng_seed(deps.storage, &prng_seed)?;

    let creator = deps.api.addr_canonicalize(&msg.creator.as_str())?;
    set_creator(deps.storage, &creator)?;

    if env.block.height > msg.deadline {
        return Err(StdError::generic_err(
            "Cannot create project with deadline in the past",
        ));
    }
    set_deadline(deps.storage, msg.deadline)?;
    set_deadman(deps.storage, msg.deadman)?;
    set_title(deps.storage, msg.title)?;
    let subtitle = msg.subtitle.unwrap_or_else(|| String::from(""));
    set_subtitle(deps.storage, subtitle)?;
    set_description(deps.storage, msg.description)?;
    let pledged_message = msg.pledged_message.unwrap_or_else(|| String::from(""));
    set_pledged_message(deps.storage, pledged_message)?;
    let funded_message = msg.funded_message.unwrap_or_else(|| String::from(""));
    set_funded_message(deps.storage, funded_message)?;
    set_reward_messages(deps.storage, msg.reward_messages)?;

    set_snip24_reward(deps.storage, deps.api, msg.snip24_reward_init)?;

    let goal = msg.goal.u128();
    if goal == 0 {
        return Err(StdError::generic_err("Goal must be greater than 0"));
    }
    set_goal(deps.storage, goal)?;

    set_categories(deps.storage, msg.categories)?;

    set_status(deps.storage, FUNDRAISING)?;
    set_total(deps.storage, 0_u128)?;

    let register_msg = PlatformExecuteMsg::Register {
        contract_addr: env.contract.address,
        contract_code_hash: env.contract.code_hash.clone(),
    };

    set_config(
        deps.storage, 
        deps.api.addr_canonicalize(msg.source_contract.as_str())?, 
        msg.source_hash.clone(),
        deps.api.addr_canonicalize(msg.snip20_contract.as_str())?,
        msg.snip20_hash.clone(),
    )?;

    let cosmos_msg = register_msg.to_cosmos_msg(msg.source_hash, msg.source_contract.into_string(), None)?;
    let snip20_register_receive_msg = register_receive_msg(
        env.contract.code_hash, 
        None, 
        256, 
        msg.snip20_hash.clone(), 
        msg.snip20_contract.clone().into_string(),
    )?;

    let viewing_key = base64::encode(&prng_seed);
    let snip20_set_viewing_key_msg = set_viewing_key_msg(
        viewing_key.clone(),
        None,
        256,
        msg.snip20_hash, 
        msg.snip20_contract.into_string(),
    )?;

    let resp = Response::new()
        .add_message(cosmos_msg)
        .add_message(snip20_register_receive_msg)
        .add_message(snip20_set_viewing_key_msg);
    Ok(resp)
}

#[entry_point]
pub fn execute(
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo, 
    msg: ExecuteMsg
) -> StdResult<Response> {
    let response = match msg {
        ExecuteMsg::ChangeText {
            title,
            subtitle,
            description,
            pledged_message,
            funded_message,
            reward_messages,
            categories,
            ..
        } => try_change_text(
            deps,
            env,
            info,
            title,
            subtitle,
            description,
            pledged_message,
            funded_message,
            reward_messages,
            categories,
        ),
        ExecuteMsg::Cancel { .. } => try_cancel(deps, env, info),
        ExecuteMsg::Receive {
            sender,
            from, 
            amount,
            msg,
        } => try_receive(deps, env, info, sender, from, amount, msg),
        ExecuteMsg::Refund { .. } => try_refund(deps, env, info),
        ExecuteMsg::PayOut { .. } => try_pay_out(deps, env, info),
        ExecuteMsg::Comment { comment, .. } => try_comment(deps, env, info, comment),
        ExecuteMsg::FlagSpam { flag, .. } => try_flag_spam(deps, env, info, flag),
        ExecuteMsg::GenerateViewingKey { entropy, .. } => {
            try_generate_viewing_key(deps, env, info, entropy)
        }
    };
    pad_response(response)
}

fn pad_response(response: StdResult<Response>) -> StdResult<Response> {
    response.map(|mut response| {
        response.data = response.data.map(|mut data| {
            space_pad(RESPONSE_BLOCK_SIZE, &mut data.0);
            data
        });
        response
    })
}

fn try_generate_viewing_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    entropy: String,
) -> StdResult<Response> {
    let prng_seed = get_prng_seed(deps.storage)?;

    let key = ViewingKey::new(&env, &info, &prng_seed, (&entropy).as_ref());

    let message_sender = deps.api.addr_canonicalize(&info.sender.as_str())?;

    write_viewing_key(deps.storage, &message_sender, &key);

    let mut resp = Response::default();
    resp.data = Some(to_binary(&ExecuteAnswer::GenerateViewingKey { key })?);
    Ok(resp)
}

fn try_change_text(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: Option<String>,
    subtitle: Option<String>,
    description: Option<String>,
    pledged_message: Option<String>,
    funded_message: Option<String>,
    reward_messages: Option<Vec<RewardMessage>>,
    categories: Option<Vec<u16>>,
) -> StdResult<Response> {
    let status;
    let msg;

    let sender_address_raw = deps.api.addr_canonicalize(&info.sender.as_str())?;
    let creator = get_creator(deps.storage)?;
    if sender_address_raw != creator {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let project_status = get_status(deps.storage)?;
    let deadline = get_deadline(deps.storage)?;

    if project_status == SUCCESSFUL || project_status == EXPIRED || is_paid_out(deps.storage) {
        status = Failure;
        msg = String::from("Cannot change a project that has been completed");
    } else if env.block.height > deadline {
        // was still FUNDRAISING but deadline expired
        set_status(deps.storage, EXPIRED)?;
        status = Failure;
        msg = String::from("Cannot change a project that has been completed");
    } else {
        let mut updates: Vec<String> = vec![];

        if title.is_some() {
            set_title(deps.storage, title.unwrap())?;
            updates.push(String::from("title"));
        }

        if subtitle.is_some() {
            set_subtitle(deps.storage, subtitle.unwrap())?;
            updates.push(String::from("subtitle"));
        }

        if description.is_some() {
            set_description(deps.storage, description.unwrap())?;
            updates.push(String::from("description"));
        }

        if pledged_message.is_some() {
            set_pledged_message(deps.storage, pledged_message.unwrap())?;
            updates.push(String::from("pledged message"));
        }

        if funded_message.is_some() {
            set_funded_message(deps.storage, funded_message.unwrap())?;
            updates.push(String::from("funded message"));
        }

        if reward_messages.is_some() {
            set_reward_messages(deps.storage, reward_messages.unwrap())?;
            updates.push(String::from("reward messages"));
        }

        if categories.is_some() {
            set_categories(deps.storage, categories.unwrap())?;
            updates.push(String::from("categories"));
        }

        if updates.len() > 0 {
            status = Success;
            msg = format!("Updated {}", updates.join(", "));
        } else {
            status = Failure;
            msg = format!("Nothing was updated");
        }
    }

    let mut resp = Response::default();
    resp.data = Some(to_binary(&ExecuteAnswer::ChangeText { status, msg })?);
    Ok(resp)
}

fn try_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {
    let status;
    let message;

    let token_address = deps.api.addr_canonicalize(&info.sender.as_str())?;
    let config = get_config(deps.storage)?;
    if token_address != config.snip20_contract {
        return Err(StdError::generic_err("Sender is incorrect SNIP-20 contract"));
    }

    let mut anonymous = false;
    
    if let Some(bin_msg) = msg {
        match from_binary(&bin_msg)? {
            ExecuteReceiveMsg::ReceiveContribution {
                anon
            } => {
                anonymous = anon
            }
        }
    }

    let project_status = get_status(deps.storage)?;
    let deadline = get_deadline(deps.storage)?;

    if project_status == EXPIRED || is_paid_out(deps.storage) {
        //TODO: change to stderror?

        status = Failure;
        message = String::from("Project is not accepting contributions")
    } else if env.block.height > deadline {
        if project_status == FUNDRAISING {
            set_status(deps.storage, EXPIRED)?;
        }
        status = Failure;
        message = String::from("Project is not accepting contributions")
    } else {
        let amount = amount.u128();

        if amount == 0 {
            status = Failure;
            message = String::from("No coins sent");
        } else {
            let total = get_total(deps.storage)?;
            let sender_address_raw = deps.api.addr_canonicalize(&info.sender.as_str())?;

            add_funds(deps.storage, &sender_address_raw, anonymous, amount)?;

            let goal = get_goal(deps.storage)?;

            if total + amount >= goal {
                set_status(deps.storage, SUCCESSFUL)?;
            }

            status = Success;
            message = format!("Successfully contributed {}", amount);
        }
    }

    let mut messages = vec![];
    if status == Failure {
        // return coins to sender
        let snip20_transfer_msg = transfer_msg(
            from.into_string(), 
            amount, 
            None, 
            None, 
            256, 
            config.snip20_hash, 
            deps.api.addr_humanize(&config.snip20_contract)?.into_string(),
        )?;
        messages.push(snip20_transfer_msg);
    }

    let mut resp = Response::new().add_messages(messages);
    resp.data = Some(to_binary(&ExecuteAnswer::Receive {
        status,
        msg: message,
    })?);
    Ok(resp)
}

fn try_cancel(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let mut response_status: ResponseStatus = Failure;
    let mut msg: String = String::from("");

    let sender_address_raw = deps.api.addr_canonicalize(&info.sender.as_str())?;
    let creator = get_creator(deps.storage)?;
    if sender_address_raw != creator {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let status = get_status(deps.storage)?;

    if status == EXPIRED {
        msg = String::from("Cannot cancel an expired project");
    } else if status == SUCCESSFUL || is_paid_out(deps.storage) {
        msg = String::from("Cannot cancel a funded project");
    } else if status == FUNDRAISING {
        response_status = Success;
        set_status(deps.storage, EXPIRED)?;
    }

    let mut resp = Response::default();
    resp.data = Some(to_binary(&ExecuteAnswer::Cancel {
        status: response_status,
        msg,
    })?);
    Ok(resp)
}

pub fn try_refund(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let response_status;
    let msg;

    let mut messages = vec![];
    let status = get_status(deps.storage)?;
    let deadline = get_deadline(deps.storage)?;
    let deadman = get_deadman(deps.storage)?;
    if is_paid_out(deps.storage) || (status == SUCCESSFUL && deadline + deadman > env.block.height) {
        response_status = Failure;
        msg = String::from("Cannot receive refund after project successfully funded");
    } else {
        let sender_address_raw = deps.api.addr_canonicalize(&info.sender.as_str())?;
        let refund_amount = clear_funds(deps.storage, &sender_address_raw)?;

        if refund_amount == 0 {
            response_status = Failure;
            msg = String::from("No funds to refund");
        } else {
            let config = get_config(deps.storage)?;
            let snip20_transfer_msg = transfer_msg(
                info.sender.into_string(), 
                Uint128::from(refund_amount), 
                None, 
                None, 
                256, 
                config.snip20_hash, 
                deps.api.addr_humanize(&config.snip20_contract)?.into_string(),
            )?;
            messages.push(snip20_transfer_msg);
            response_status = Success;
            msg = format!("{} refunded", refund_amount);
        }
    }

    let mut resp = Response::new().add_messages(messages);
    resp.data = Some(to_binary(&ExecuteAnswer::Refund {
        status: response_status,
        msg,
    })?);
    Ok(resp)
}

fn try_pay_out(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let response_status;
    let msg;

    let sender_address_raw = deps.api.addr_canonicalize(&info.sender.as_str())?;
    let creator = get_creator(deps.storage)?;
    if sender_address_raw != creator {
        return Err(StdError::generic_err("Unauthorized"));
    }

    if is_paid_out(deps.storage) {
        return Err(StdError::generic_err("Already paid out"));
    }

    let mut messages = vec![];
    let status = get_status(deps.storage)?;
    let deadline = get_deadline(deps.storage)?;
    let deadman = get_deadman(deps.storage)?;

    // time has completed and it is successful
    //   and deadman time has not elapsed
    if env.block.height > deadline && status == SUCCESSFUL {
        if deadline + deadman < env.block.height {
            response_status = Failure;
            msg = String::from(
                "Project was funded, but the deadman time has elapsed and funding has expired"
            );
        } else {
            let total = get_total(deps.storage)?;
            let config = get_config(deps.storage)?;
            let snip20_transfer_msg = transfer_msg(
                info.sender.into_string(), 
                Uint128::from(total), 
                None, 
                None, 
                256, 
                config.snip20_hash, 
                deps.api.addr_humanize(&config.snip20_contract)?.into_string(),
            )?;
            messages.push(snip20_transfer_msg);
            msg = format!("Pay out {} uscrt", total);
    
            paid_out(deps.storage)?;
            response_status = Success;
        }
    } else {
        if env.block.height > deadline && status == FUNDRAISING {
            set_status(deps.storage, EXPIRED)?;
        }
        response_status = Failure;
        msg = String::from(
            "Cannot receive pay out unless project successfully funded and deadline past",
        );
    }

    let mut resp = Response::new().add_messages(messages);
    resp.data = Some(to_binary(&ExecuteAnswer::PayOut {
        status: response_status,
        msg,
    })?);
    Ok(resp)
}

pub fn try_comment(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    comment: String,
) -> StdResult<Response> {
    let mut response_status = Failure;
    let msg;

    let status = get_status(deps.storage)?;

    if status == EXPIRED {
        msg = String::from("Cannot comment on an expired project");
    } else if status == SUCCESSFUL || is_paid_out(deps.storage) {
        msg = String::from("Cannot comment on a funded project");
    } else {
        push_comment(deps.storage, comment)?;
        msg = String::from("Comment added");
        response_status = Success;
    }

    let mut resp = Response::default();
    resp.data = Some(to_binary(&ExecuteAnswer::Cancel {
        status: response_status,
        msg,
    })?);
    Ok(resp)
}

pub fn try_flag_spam(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    flag: bool,
) -> StdResult<Response> {
    set_spam_flag(deps.storage, &deps.api.addr_canonicalize(&info.sender.as_str())?, flag)?;
    let spam_count = get_spam_count(deps.storage)?;
    let mut resp = Response::default();
    resp.data = Some(to_binary(&ExecuteAnswer::FlagSpam {
        spam_count,
        status: Success,
        msg: String::from("Flag spam updated"),
    })?);
    Ok(resp)
}

#[entry_point]
pub fn query(
    deps: Deps, 
    _env: Env, 
    msg: QueryMsg
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Status {} => query_status(deps),
        QueryMsg::StatusWithPermit { permit } => query_status_with_permit(deps, &permit),
        QueryMsg::Comments { page, page_size } => query_comments(deps, page, page_size),
        _ => authenticated_queries(deps, msg),
    }
}

fn authenticated_queries(
    deps: Deps,
    msg: QueryMsg,
) -> StdResult<Binary> {
    let (addresses, key) = msg.get_validation_params();

    for address in addresses {
        let canonical_addr = deps.api.addr_canonicalize(address.as_str())?;

        let expected_key = read_viewing_key(deps.storage, &canonical_addr);

        if expected_key.is_none() {
            // Checking the key will take significant time. We don't want to exit immediately if it isn't set
            // in a way which will allow to time the command and determine if a viewing key doesn't exist
            key.check_viewing_key(&[0u8; VIEWING_KEY_SIZE]);
        } else if key.check_viewing_key(expected_key.unwrap().as_slice()) {
            return match msg {
                // Base
                QueryMsg::StatusAuth { address, .. } => query_status_auth(deps, &address),
                _ => panic!("This query type does not require authentication"),
            };
        }
    }

    Err(StdError::generic_err("Unauthorized"))
}

fn query_comments(deps: Deps, page: u32, page_size: u32) -> StdResult<Binary> {
    let comments = get_comments(deps.storage, page, page_size)?;
    to_binary(&QueryAnswer::Comments { comments })
}

fn query_status(deps: Deps) -> StdResult<Binary> {
    let status_string;

    let status = get_status(deps.storage)?;
    if status == FUNDRAISING {
        status_string = String::from("fundraising");
    } else if status == EXPIRED {
        status_string = String::from("expired");
    } else if status == SUCCESSFUL {
        status_string = String::from("successful");
    } else {
        return Err(StdError::generic_err("Error getting status"));
    }

    let creator = get_creator(deps.storage)?;
    let creator = deps.api.addr_humanize(&creator)?;

    let po = is_paid_out(deps.storage);

    let goal = get_goal(deps.storage)?;
    let goal = Uint128::from(goal);

    let total = get_total(deps.storage)?;
    let total = Uint128::from(total);

    let deadline = get_deadline(deps.storage)?;
    let deadman = get_deadman(deps.storage)?;

    let title = get_title(deps.storage);

    let subtitle = get_subtitle(deps.storage);

    let description = get_description(deps.storage);

    let categories = get_categories(deps.storage)?;

    let spam_count = get_spam_count(deps.storage)?;

    to_binary(&QueryAnswer::Status {
        creator,
        status: status_string,
        paid_out: po,
        goal,
        total,
        deadline,
        deadman,
        title,
        subtitle,
        description,
        categories,
        spam_count,
    })
}

fn query_status_auth(
    deps: Deps,
    address: &Addr,
) -> StdResult<Binary> {
    let status_string;

    let status = get_status(deps.storage)?;

    if status == FUNDRAISING {
        status_string = String::from("fundraising");
    } else if status == EXPIRED {
        status_string = String::from("expired");
    } else if status == SUCCESSFUL {
        status_string = String::from("successful");
    } else {
        return Err(StdError::generic_err("Error getting status"));
    }

    let creator = get_creator(deps.storage)?;
    let creator = deps.api.addr_humanize(&creator)?;

    let po = is_paid_out(deps.storage);

    let goal = get_goal(deps.storage)?;
    let goal = Uint128::from(goal);

    let total = get_total(deps.storage)?;
    let total = Uint128::from(total);

    let deadline = get_deadline(deps.storage)?;
    let deadman = get_deadman(deps.storage)?;

    let title = get_title(deps.storage);
    let subtitle = get_subtitle(deps.storage);
    let description = get_description(deps.storage);

    let categories = get_categories(deps.storage)?;

    let spam_count = get_spam_count(deps.storage)?;

    let sender_address_raw = deps.api.addr_canonicalize(&address.as_str())?;
    let stored_funder = get_funder(deps.storage, &sender_address_raw);

    let mut pledged_message: Option<String> = None;
    let mut funded_message: Option<String> = None;
    let mut reward_messages: Vec<RewardMessage> = vec![];
    let mut contribution: Option<Uint128> = None;

    match stored_funder {
        Ok(stored_funder) => {
            if stored_funder.amount > 0 {
                if status != EXPIRED {
                    pledged_message = Some(get_pledged_message(deps.storage));
                }
                if status == SUCCESSFUL && is_paid_out(deps.storage) {
                    funded_message = Some(get_funded_message(deps.storage));
                    reward_messages = get_reward_messages(deps.storage)?
                        .into_iter()
                        .filter(|reward_message| {
                            stored_funder.amount >= reward_message.threshold.u128()
                        })
                        .collect();
                }
            }
            contribution = Some(Uint128::from(stored_funder.amount));
        }
        Err(_) => {}
    };

    to_binary(&QueryAnswer::StatusAuth {
        creator,
        status: status_string,
        paid_out: po,
        goal,
        total,
        deadline,
        deadman,
        title,
        subtitle,
        description,
        categories,
        spam_count,
        pledged_message,
        funded_message,
        reward_messages,
        contribution,
    })
}

fn query_status_with_permit(
    deps: Deps,
    permit: &Permit,
) -> StdResult<Binary> {
    let get_validate_permit = PlatformQueryMsg::ValidatePermit { permit: permit.clone() };
    let config = get_config(deps.storage)?;
    let validate_permit_response: ValidatePermitResponse = get_validate_permit.query(
        deps.querier,
        config.platform_hash.to_string(),
        deps.api.addr_humanize(&config.platform_contract)?.into_string(),
    )?;
    let address = validate_permit_response.validate_permit.address;

    query_status_auth(deps, &address)
}
