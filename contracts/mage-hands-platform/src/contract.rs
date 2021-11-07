use crate::msg::{
    ContractInfo, HandleAnswer, HandleMsg, InitMsg, ProjectInitMsg, QueryAnswer, QueryMsg,
    ResponseStatus::Failure, ResponseStatus::Success,
};
use crate::state::{
    add_project, get_config, get_projects, get_projects_count, is_creating_project, project_count,
    set_config, set_creating_project, Config, Fee, StoredContractInfo,
};
use cosmwasm_std::{
    to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    InitResponse, Querier, QueryResult, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::utils::InitCallback;

const DENOM: &str = "uscrt";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let owner;
    if msg.owner.is_some() {
        owner = deps.api.canonical_address(&msg.owner.unwrap())?;
    } else {
        owner = deps.api.canonical_address(&env.message.sender)?;
    }

    set_config(
        &mut deps.storage,
        owner,
        msg.default_upfront.u128(),
        msg.default_fee.into_stored()?,
        msg.project_contract_code_id,
        msg.project_contract_code_hash.as_bytes().to_vec(),
    )?;

    //debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Create {
            title,
            subtitle,
            description,
            pledged_message,
            funded_message,
            goal,
            deadline,
            categories,
            entropy,
            ..
        } => try_create(
            deps,
            env,
            title,
            subtitle,
            description,
            pledged_message,
            funded_message,
            goal,
            deadline,
            categories,
            entropy,
        ),
        HandleMsg::Config {
            owner,
            default_upfront,
            default_fee,
            project_contract_code_id,
            project_contract_code_hash,
            ..
        } => try_config(
            deps,
            env,
            owner,
            default_upfront,
            default_fee,
            project_contract_code_id,
            project_contract_code_hash,
        ),
        HandleMsg::Register {
            contract_addr,
            contract_code_hash,
        } => try_register(deps, env, contract_addr, contract_code_hash),
    }
}

pub fn try_create<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    title: String,
    subtitle: Option<String>,
    description: String,
    pledged_message: Option<String>,
    funded_message: Option<String>,
    goal: Uint128,
    deadline: u64,
    categories: Vec<u16>,
    entropy: String,
) -> StdResult<HandleResponse> {
    let status;
    let msg;

    let sent_coins = env.message.sent_funds.clone();
    let config: Config = get_config(&deps.storage)?;
    let mut messages = vec![];

    if sent_coins[0].denom != DENOM {
        // sent wrong kind of coins
        // send them back
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address.clone(),
            to_address: env.message.sender,
            amount: env.message.sent_funds,
        }));

        status = Failure;
        msg = String::from("Wrong denomination");
    } else if sent_coins[0].amount.u128() != config.default_upfront {
        // incorrect amount sent
        // send them back
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address.clone(),
            to_address: env.message.sender,
            amount: env.message.sent_funds,
        }));
        status = Failure;
        msg = format!(
            "Upfront fee not correct, should be {} uscrt",
            config.default_upfront
        );
    } else {
        set_creating_project(&mut deps.storage, true)?;

        let project_init_msg = ProjectInitMsg {
            creator: env.message.sender,
            title,
            subtitle,
            description,
            pledged_message,
            funded_message,
            goal,
            deadline,
            categories,
            commission_addr: deps.api.human_address(&config.owner)?,
            upfront: Uint128(config.default_upfront),
            fee: config.default_fee.into_humanized()?,
            entropy,
            source_contract: env.contract.address.clone(),
            source_hash: env.contract_code_hash,
            padding: None,
        };
        let label = format!(
            "{}-Mage-Hands-Project-{}-{}",
            &env.contract.address.clone(),
            project_count(&deps.storage)?,
            &base64::encode(env.block.time.to_be_bytes()),
        );

        let config: Config = get_config(&deps.storage)?;

        let cosmos_msg = project_init_msg.to_cosmos_msg(
            label.clone(),
            config.project_contract_code_id,
            String::from_utf8(config.project_contract_code_hash).unwrap_or_default(),
            None,
        )?;
        messages.push(cosmos_msg);

        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address.clone(),
            to_address: deps.api.human_address(&config.owner)?,
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: sent_coins[0].amount,
            }],
        }));

        status = Success;
        msg = format!("Created project contract {}", label);
    }

    //debug_print("created new project successfully");
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Create { status, msg })?),
    })
}

fn try_register<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    contract_addr: HumanAddr,
    contract_code_hash: String,
) -> StdResult<HandleResponse> {
    if !is_creating_project(&deps.storage) {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    set_creating_project(&mut deps.storage, false)?;
    let contract_info = StoredContractInfo {
        address: deps.api.canonical_address(&contract_addr)?,
        code_hash: contract_code_hash.clone(),
    };
    let project_id = add_project(&mut deps.storage, contract_info)?;

    let status = Success;
    let msg = format!("Registered contract {}", contract_addr);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Register {
            status,
            msg,
            project_id,
            project_address: contract_addr,
            project_code_hash: contract_code_hash,
        })?),
    })
}

fn try_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
    default_upfront: Option<Uint128>,
    default_fee: Option<Fee>,
    project_contract_code_id: Option<u64>,
    project_contract_code_hash: Option<String>,
) -> StdResult<HandleResponse> {
    let status;
    let msg;

    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
    let mut config = get_config(&deps.storage)?;

    if sender_address_raw != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    if owner.is_some() {
        config.owner = deps.api.canonical_address(&owner.unwrap())?;
    }

    if default_upfront.is_some() {
        config.default_upfront = default_upfront.unwrap().u128();
    }

    if default_fee.is_some() {
        config.default_fee = default_fee.unwrap().into_stored()?;
    }

    if project_contract_code_id.is_some() {
        config.project_contract_code_id = project_contract_code_id.unwrap();
    }

    if project_contract_code_hash.is_some() {
        config.project_contract_code_hash = project_contract_code_hash.unwrap().as_bytes().to_vec();
    }

    set_config(
        &mut deps.storage,
        config.owner.clone(),
        config.default_upfront,
        config.default_fee.clone(),
        config.project_contract_code_id.clone(),
        config.project_contract_code_hash.clone(),
    )?;

    status = Success;
    msg = format!(
        "New config: owner {}, default_upfront {}, default_fee {}/{}, project code id {}, project code hash {}", 
        config.owner,
        config.default_upfront,
        config.default_fee.commission_rate_nom,
        config.default_fee.commission_rate_denom,
        config.project_contract_code_id,
        String::from_utf8(config.project_contract_code_hash).unwrap_or_default(),
    );

    //debug_print("config set successfully");
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Config { status, msg })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Projects { page, page_size } => query_projects(deps, page, page_size),
    }
}

fn query_projects<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    page: u32,
    page_size: u32,
) -> QueryResult {
    let mut count = 0_u32;
    if page_size < 1 {
        return Err(StdError::generic_err("Invalid page_size"));
    }
    let mut projects: Vec<ContractInfo> = vec![];
    let projects_wrapped = get_projects(&deps.storage, page, page_size);
    if projects_wrapped.is_ok() {
        projects = projects_wrapped
            .unwrap()
            .iter()
            .map(|project| project.to_humanized(&deps.api).unwrap())
            .collect();
        count = get_projects_count(&deps.storage)?;
    }
    to_binary(&QueryAnswer::Projects { projects, count })
}
