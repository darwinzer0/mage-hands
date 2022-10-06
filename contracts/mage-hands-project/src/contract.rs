use std::cmp::{min,max};
use primitive_types::U256;
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Env, Addr,
    Response, StdError, StdResult, Uint128, DepsMut, Deps, MessageInfo,
    WasmMsg, SubMsg, CosmosMsg, Reply, CanonicalAddr,
    Storage, Api,
};
use rand::RngCore;
use crate::msg::{
    ExecuteAnswer, ExecuteMsg, InstantiateMsg, PlatformExecuteMsg, QueryAnswer, QueryMsg, ResponseStatus,
    ResponseStatus::Failure, ResponseStatus::Success, PlatformQueryMsg, ValidatePermitResponse,
    ExecuteReceiveMsg,
};
use crate::reward::{RewardMessage, Snip24InstantiateMsg, InitConfig, InitialBalance, Snip24RewardInit, VestingReward, VestingRewardStatus};
use crate::state::{
    get_subtitle, set_subtitle,
    add_funds, clear_funds, get_categories, get_creator, get_deadline,
    get_description, get_funded_message, get_funder, get_goal, get_pledged_message,
    get_prng_seed, get_status, get_title, get_total, is_paid_out, paid_out,
    read_viewing_key, set_categories, set_creator, set_deadline,
    set_description, set_funded_message, set_goal, set_pledged_message, set_prng_seed,
    set_status, set_title, set_total, write_viewing_key, EXPIRED, FUNDRAISING,
    SUCCESSFUL, set_config, get_config, set_deadman, get_deadman,
    push_comment, get_comments, set_spam_flag, get_spam_count, set_snip24_reward, set_reward_messages, 
    get_reward_messages, get_snip24_reward, set_snip24_reward_address, get_snip24_reward_address, set_creator_snip24_allocation_received, get_creator_snip24_allocation_received, set_funder,
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
use crate::random::{supply_more_entropy, get_random_number_generator};
use crate::parse_reply::{parse_reply_instantiate_data};

pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";
pub const RESPONSE_BLOCK_SIZE: usize = 256;
pub const SNIP24_INSTANTIATE_REPLY_ID: u64 = 1;
pub const PER_MILLE_DENOM: u16 = 1000;

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

    validate_snip24_reward_init(msg.snip24_reward_init.clone())?;
    set_snip24_reward(deps.storage, deps.api, msg.snip24_reward_init.clone())?;
    let snip24_allocation_received: Vec<bool>;
    if msg.snip24_reward_init.is_none() {
        snip24_allocation_received = vec![];
    } else {
        let snip24_reward_init = msg.snip24_reward_init.unwrap();
        if snip24_reward_init.contributors_vesting_schedule.len() <= 1 {
            snip24_allocation_received = vec![false];
        } else {
            snip24_allocation_received = snip24_reward_init.contributors_vesting_schedule.into_iter().map(|_| false).collect();
        }
    }
    set_creator_snip24_allocation_received(deps.storage, snip24_allocation_received)?;

    set_snip24_reward_address(deps.storage, None)?;

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

fn validate_snip24_reward_init(
    reward_init: Option<Snip24RewardInit>,
) -> StdResult<()> {
    if reward_init.is_some() {
        let reward_init = reward_init.unwrap();
        let sum = reward_init.contributors_vesting_schedule
            .into_iter()
            .map(|r| r.per_mille)
            .reduce(|accum, item| { accum + item });
        match sum {
            Some(PER_MILLE_DENOM) => {},
            None => {},
            _ => { return Err(StdError::generic_err(format!("Contributors' vesting schedule per mille does not sum to {}", PER_MILLE_DENOM))); },
        }

        let sum = reward_init.creator_vesting_schedule
            .into_iter()
            .map(|r| r.per_mille)
            .reduce(|accum, item| { accum + item });
        match sum {
            Some(PER_MILLE_DENOM) => {},
            None => {},
            _ => { return Err(StdError::generic_err(format!("Creator's vesting schedule per mille does not sum to {}", PER_MILLE_DENOM))); },
        }
    }
    Ok(())
}

#[entry_point]
pub fn execute(
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo, 
    msg: ExecuteMsg
) -> StdResult<Response> {
    let mut fresh_entropy = to_binary(&msg)?.0;
    fresh_entropy.extend(to_binary(&env)?.0);
    fresh_entropy.extend(to_binary(&info)?.0);
    supply_more_entropy(deps.storage, fresh_entropy.as_slice())?;
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
        ExecuteMsg::ClaimReward { idx, .. } => try_claim_reward(deps, env, info, idx),
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
            let sender_address_raw = deps.api.addr_canonicalize(&from.as_str())?;

            // make sure it is not the project creator
            if sender_address_raw == get_creator(deps.storage)? {
                return Err(StdError::generic_err("Cannot fund your own project"));
            }

            let snip24_reward_init = get_snip24_reward(deps.storage, deps.api)?;
            let snip24_rewards_received: Vec<bool>;
            if snip24_reward_init.is_none() {
                snip24_rewards_received = vec![];
            } else {
                let snip24_reward_init = snip24_reward_init.unwrap();
                if snip24_reward_init.contributors_vesting_schedule.len() <= 1 {
                    snip24_rewards_received = vec![false];
                } else {
                    snip24_rewards_received = snip24_reward_init.contributors_vesting_schedule.into_iter().map(|_| false).collect();
                }
            }
            add_funds(deps.storage, &sender_address_raw, anonymous, amount, snip24_rewards_received)?;

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

    let mut transfer_message: Option<CosmosMsg> = None;
    let mut instantiate_message: Option<CosmosMsg> = None;
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
            transfer_message = Some(transfer_msg(
                info.sender.clone().into_string(), 
                Uint128::from(total), 
                None, 
                None, 
                256, 
                config.snip20_hash, 
                deps.api.addr_humanize(&config.snip20_contract)?.into_string(),
            )?);
            msg = format!("Pay out {} tokens", total);
    
            paid_out(deps.storage)?;

            // handle snip24 reward

            // create the snip24 contract and allocate coins to project contract
            // all vesting will be handled by this contract
            let snip24_reward_init = get_snip24_reward(deps.storage, deps.api)?;
            if snip24_reward_init.is_some() {
                let snip24_reward_init = snip24_reward_init.unwrap();

                // Creating a message to create new snip24 token
                instantiate_message = Some(CosmosMsg::Wasm(WasmMsg::Instantiate {
                    code_id: snip24_reward_init.reward_snip24_code_id,
                    code_hash: snip24_reward_init.reward_snip24_code_hash,
                    msg: to_binary(&Snip24InstantiateMsg {
                        admin: snip24_reward_init.admin,
                        name: snip24_reward_init.name.clone(),
                        symbol: snip24_reward_init.symbol.clone(),
                        decimals: snip24_reward_init.decimals,
                        initial_balances: Some(vec![
                            InitialBalance {
                                // allocate all coins to the project contract
                                address: env.contract.address.clone(),
                                amount: snip24_reward_init.amount,
                            }
                        ]),
                        config: Some(InitConfig {
                            public_total_supply: Some(snip24_reward_init.public_total_supply),
                            enable_deposit: Some(snip24_reward_init.enable_deposit),
                            enable_redeem: Some(snip24_reward_init.enable_redeem),
                            enable_mint: Some(snip24_reward_init.enable_mint),
                            enable_burn: Some(snip24_reward_init.enable_burn),
                        }),
                        prng_seed: to_binary(
                            &sha_256(
                                [
                                    &get_random_number_generator(deps.storage).next_u64().to_be_bytes(), 
                                    to_binary(&env)?.0.as_slice(),
                                    to_binary(&info)?.0.as_slice(),
                                ].concat().as_slice()
                            )
                        )?, 
                    })?,
                    funds: vec![],
                    label: format!("{}-{}-{}", snip24_reward_init.name, snip24_reward_init.symbol, env.block.height),
                }));
            }

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

    let mut submessages: Vec<SubMsg> = vec![];
    if instantiate_message.is_some() {
        let instantiate_submsg = SubMsg::reply_on_success(instantiate_message.unwrap(), SNIP24_INSTANTIATE_REPLY_ID);
        submessages.push(instantiate_submsg);
    }
    if transfer_message.is_some() {
        submessages.push(SubMsg::new(transfer_message.unwrap()));
    }
    let mut resp = Response::new().add_submessages(submessages);
    resp.data = Some(to_binary(&ExecuteAnswer::PayOut {
        status: response_status,
        msg,
    })?);
    Ok(resp)
}

fn calculate_contributor_snip24_rewards(
    storage: &dyn Storage,
    api: &dyn Api,
    address: CanonicalAddr,
) -> StdResult<Vec<VestingReward>> {
    let result: Vec<VestingReward>;
    let snip24_reward_init = get_snip24_reward(storage, api)?;
    match snip24_reward_init {
        Some(snip24_reward_init) => { 
            let funder = get_funder(storage, &address)?;
            let valid_amount: u128;
            if funder.amount < snip24_reward_init.minimum_contribution.u128() {
                result = vec![];
            } else {
                let max_contribution = snip24_reward_init.maximum_contribution.u128();
                if max_contribution == 0_u128 {
                    valid_amount = funder.amount;
                } else {
                    valid_amount = min(funder.amount, max_contribution);
                }

                let total = get_total(storage)?;

                // assume linear allocation (TODO: others)
                let total_reward_u256: U256 = U256::from(snip24_reward_init.amount.u128())
                    .checked_mul(U256::from(snip24_reward_init.contributors_per_mille)).expect("Overflow when calculating reward")
                    .checked_div(U256::from(PER_MILLE_DENOM)).expect("Div by zero when calculating reward")
                    .checked_mul(U256::from(valid_amount)).expect("Overflow when calculating reward")
                    .checked_div(U256::from(total)).expect("Div by zero when calculating reward");

                // no vesting schedule for contributors
                if snip24_reward_init.contributors_vesting_schedule.len() == 0 {
                    result = vec![
                        VestingReward {
                            block: 0_u64,
                            amount: total_reward_u256.as_u128(),
                        }
                    ];
                } else {
                    result = snip24_reward_init.contributors_vesting_schedule
                        .into_iter()
                        .map(|event| {
                            let partial_reward_u256: U256 = U256::from(total_reward_u256)
                                .checked_mul(U256::from(event.per_mille)).expect("Overflow when calculating reward")
                                .checked_div(U256::from(PER_MILLE_DENOM)).expect("Div by zero when calculating reward");
                            VestingReward {
                                block: event.block,
                                amount: partial_reward_u256.as_u128(),
                            }
                        })
                        .collect();
                }
            }
        },
        None => { result = vec![]; }
    };
    Ok(result)
}

fn calculate_creator_snip24_allocation(
    storage: &dyn Storage,
    api: &dyn Api,
) -> StdResult<Vec<VestingReward>> {
    let result: Vec<VestingReward>;
    let snip24_reward_init = get_snip24_reward(storage, api)?;
    match snip24_reward_init {
        Some(snip24_reward_init) => { 
            let total_allocation_u256: U256 = U256::from(snip24_reward_init.amount.u128())
                .checked_mul(U256::from(snip24_reward_init.creator_per_mille)).expect("Overflow when calculating reward")
                .checked_div(U256::from(PER_MILLE_DENOM)).expect("Div by zero when calculating reward");
            
            if snip24_reward_init.creator_vesting_schedule.len() == 0 {
                result = vec![
                    VestingReward {
                        block: 0_u64,
                        amount: total_allocation_u256.as_u128(),
                    }
                ];
            } else {
                result = snip24_reward_init.creator_vesting_schedule
                .into_iter()
                .map(|event| {
                    let partial_reward_u256: U256 = U256::from(total_allocation_u256)
                        .checked_mul(U256::from(event.per_mille)).expect("Overflow when calculating reward")
                        .checked_div(U256::from(PER_MILLE_DENOM)).expect("Div by zero when calculating reward");
                    VestingReward {
                        block: event.block,
                        amount: partial_reward_u256.as_u128(),
                    }
                })
                .collect();
            }
        },
        None => { result = vec![]; }
    };
    Ok(result)
}

fn try_claim_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    idx: u32,
) -> StdResult<Response> {
    let response_status;
    let msg;
    let idx: usize = idx as usize;

    let mut transfer_message: Option<CosmosMsg> = None;
    let status = get_status(deps.storage)?;
    if is_paid_out(deps.storage) {
        let sender_address_raw = deps.api.addr_canonicalize(&info.sender.as_str())?;
        let is_creator = get_creator(deps.storage)? == sender_address_raw;
        if is_creator {
            let creator_allocation = calculate_creator_snip24_allocation(deps.storage, deps.api)?;
            let mut allocation_received = get_creator_snip24_allocation_received(deps.storage)?;
            let snip24_rewards: Vec<VestingRewardStatus> = creator_allocation
                .into_iter()
                .enumerate()
                .map(|(index, reward)| {
                    VestingRewardStatus { 
                        amount: Uint128::from(reward.amount), 
                        block: reward.block, 
                        received: allocation_received[index],
                    }
                })
                .collect();
            if snip24_rewards[idx].received {
                response_status = Failure;
                msg = String::from("Already claimed reward");
            } else if env.block.height < snip24_rewards[idx].block {
                response_status = Failure;
                msg = String::from("Vesting time has not been reached");
            } else {
                let config = get_config(deps.storage)?;
                transfer_message = Some(transfer_msg(
                    info.sender.clone().into_string(), 
                    Uint128::from(snip24_rewards[idx].amount.u128()), 
                    None, 
                    None, 
                    256, 
                    config.snip20_hash, 
                    deps.api.addr_humanize(&config.snip20_contract)?.into_string(),
                )?);
 
                allocation_received[idx] = true;
                set_creator_snip24_allocation_received(deps.storage, allocation_received)?;

                response_status = Success;
                msg = format!("Receive {} tokens", snip24_rewards[idx].amount.u128());
            }
        } else { // !is_creator
            let mut funder = get_funder(deps.storage, &sender_address_raw)?;
            let contributor_reward = calculate_contributor_snip24_rewards(deps.storage, deps.api, sender_address_raw.clone())?;
            let snip24_rewards: Vec<VestingRewardStatus> = contributor_reward
                .into_iter()
                .enumerate()
                .map(|(index, reward)| {
                    VestingRewardStatus {
                        amount: Uint128::from(reward.amount),
                        block: reward.block,
                        received: funder.snip24_rewards_received[index],
                    }
                })
                .collect();
            
            if snip24_rewards[idx].received {
                response_status = Failure;
                msg = String::from("Already claimed reward");
            } else if env.block.height < snip24_rewards[idx].block {
                response_status = Failure;
                msg = String::from("Vesting time has not been reached");
            } else {
                let config = get_config(deps.storage)?;
                transfer_message = Some(transfer_msg(
                    info.sender.clone().into_string(), 
                    Uint128::from(snip24_rewards[idx].amount.u128()), 
                    None, 
                    None, 
                    256, 
                    config.snip20_hash, 
                    deps.api.addr_humanize(&config.snip20_contract)?.into_string(),
                )?);

                funder.snip24_rewards_received[idx] = true;
                set_funder(
                    deps.storage, 
                    &sender_address_raw, 
                    funder.idx, 
                    funder.anonymous, 
                    funder.amount, 
                    funder.snip24_rewards_received
                )?;

                response_status = Success;
                msg = format!("Receive {} tokens", snip24_rewards[idx].amount.u128());
            }
        }
    } else {
        response_status = Failure;
        msg = String::from("Cannot claim reward");
    }

    let mut submessages: Vec<SubMsg> = vec![];
    if transfer_message.is_some() {
        submessages.push(SubMsg::new(transfer_message.unwrap()));
    }
    let mut resp = Response::new().add_submessages(submessages);
    resp.data = Some(to_binary(&ExecuteAnswer::ClaimReward {
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
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        SNIP24_INSTANTIATE_REPLY_ID => handle_instantiate_reply(deps, msg),
        id => Err(StdError::generic_err(format!("Unknown reply id: {}", id))),
    }
}

fn handle_instantiate_reply(deps: DepsMut, msg: Reply) -> StdResult<Response> {
    // Handle the msg data and save the contract address
    // See: https://github.com/CosmWasm/cw-plus/blob/main/packages/utils/src/parse_reply.rs
    let res = parse_reply_instantiate_data(msg);
    if res.is_ok() {
        let res = res.unwrap();
        // Save res.contract_address
        set_snip24_reward_address(deps.storage, Some(deps.api.addr_canonicalize(&res.contract_address)?))?;
    } else {
        let err = res.err().unwrap().to_string();
        return Err(StdError::generic_err(err));
    }

    Ok(Response::new())
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

    let sender_address_raw = deps.api.addr_canonicalize(&address.as_str())?;
    let creator = get_creator(deps.storage)?;
    let is_creator = creator == sender_address_raw;

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

    let stored_funder = get_funder(deps.storage, &sender_address_raw);

    let mut pledged_message: Option<String> = None;
    let mut funded_message: Option<String> = None;
    let mut reward_messages: Vec<RewardMessage> = vec![];
    let mut snip24_rewards: Vec<VestingRewardStatus> = vec![];
    let mut contribution: Option<Uint128> = None;

    if is_creator {
        pledged_message = Some(get_pledged_message(deps.storage));
        funded_message = Some(get_funded_message(deps.storage));
        reward_messages = get_reward_messages(deps.storage)?;

        let creator_allocation = calculate_creator_snip24_allocation(deps.storage, deps.api)?;
        let allocation_received = get_creator_snip24_allocation_received(deps.storage)?;
        snip24_rewards = creator_allocation
            .into_iter()
            .enumerate()
            .map(|(idx, reward)| {
                VestingRewardStatus { 
                    amount: Uint128::from(reward.amount), 
                    block: reward.block, 
                    received: allocation_received[idx],
                }
            })
            .collect();
    } else {
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

                let contributor_rewards = calculate_contributor_snip24_rewards(deps.storage, deps.api, sender_address_raw)?;
                snip24_rewards = contributor_rewards
                    .into_iter()
                    .enumerate()
                    .map(|(idx, reward)| {
                        VestingRewardStatus { 
                            amount: Uint128::from(reward.amount), 
                            block: reward.block, 
                            received: stored_funder.snip24_rewards_received[idx],
                        }
                    })
                    .collect();
            }
            Err(_) => {}
        };
    }

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
        snip24_rewards,
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
