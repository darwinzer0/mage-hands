use crate::msg::{
    ContractInfo, ExecuteAnswer, ExecuteMsg, InstantiateMsg, ProjectInstantiateMsg, QueryAnswer, QueryMsg, ResponseStatus::Success, space_pad,
};
use crate::state::{
    add_project, get_config, get_projects, is_creating_project, project_count,
    set_config, set_creating_project, Config, StoredContractInfo,
};
use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Env, DepsMut, MessageInfo, Addr,
    Response, StdError, StdResult, Deps, Uint128, SubMsg,
};
use secret_toolkit::utils::{ InitCallback, };
use secret_toolkit::permit::{ validate, RevokedPermits,Permit, };

const DENOM: &str = "uscrt";
pub const RESPONSE_BLOCK_SIZE: usize = 256;
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let owner;
    if msg.owner.is_some() {
        owner = deps.api.addr_canonicalize(msg.owner.unwrap().as_str())?;
    } else {
        owner = deps.api.addr_canonicalize(info.sender.as_str())?;
    }

    set_config(
        deps.storage,
        owner,
        msg.project_contract_code_id,
        msg.project_contract_code_hash.as_bytes().to_vec(),
        deps.api.addr_canonicalize(env.contract.address.as_str())?,
    )?;

    Ok(Response::new().add_attribute("init", "😎"))
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

#[entry_point]
pub fn execute(
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo, 
    msg: ExecuteMsg
) -> StdResult<Response> {
    let response = match msg {
        ExecuteMsg::Create {
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
            info,
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
        ExecuteMsg::Config {
            owner,
            project_contract_code_id,
            project_contract_code_hash,
            ..
        } => try_config(
            deps,
            env,
            info,
            owner,
            project_contract_code_id,
            project_contract_code_hash,
        ),
        ExecuteMsg::Register {
            contract_addr,
            contract_code_hash,
        } => try_register(deps, env, info, contract_addr, contract_code_hash),
        ExecuteMsg::RevokePermit { permit_name, .. } => revoke_permit(deps, env, info, permit_name),
    };
    pad_response(response)
}

pub fn try_create(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    subtitle: Option<String>,
    description: String,
    pledged_message: Option<String>,
    funded_message: Option<String>,
    goal: Uint128,
    deadline: u64,
    categories: Vec<u16>,
    entropy: String,
) -> StdResult<Response> {
    let msg;

    let sent_coins = info.funds.clone();
    let config: Config = get_config(deps.storage)?;
    let mut messages = vec![];

    if sent_coins[0].denom != DENOM {
        // sent wrong kind of coins
        return Err(StdError::generic_err("Incorrect coin"));
    } else {
        set_creating_project(deps.storage, true)?;

        let project_init_msg = ProjectInstantiateMsg {
            creator: info.sender,
            title,
            subtitle,
            description,
            pledged_message,
            funded_message,
            goal,
            deadline,
            categories,
            entropy,
            source_contract: env.contract.address.clone(),
            source_hash: env.contract.code_hash,
            padding: None,
        };
        let label = format!(
            "{}-Mage-Hands-Project-{}-{}",
            &env.contract.address.clone(),
            project_count(deps.storage)?,
            &base64::encode(env.block.time.to_string()),
        );

        let config: Config = get_config(deps.storage)?;

        let cosmos_msg = SubMsg::new(project_init_msg.to_cosmos_msg(
            label.clone(),
            config.project_contract_code_id,
            String::from_utf8(config.project_contract_code_hash).unwrap_or_default(),
            None,
        )?);
        messages.push(cosmos_msg);

        messages.push(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: deps.api.addr_humanize(&config.owner)?.into_string(),
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: sent_coins[0].amount,
            }],
        })));

        msg = format!("Created project contract {}", label);
    }

    let mut resp = Response::default();
    resp.messages = messages;
    resp.data = Some(to_binary(&ExecuteAnswer::Create {
        status: Success,
        msg,
    })?);
    Ok(resp)
}

fn try_register(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    contract_addr: Addr,
    contract_code_hash: String,
) -> StdResult<Response> {
    if !is_creating_project(deps.storage) {
        return Err(StdError::generic_err(format!("Unauthorized")));
    }

    set_creating_project(deps.storage, false)?;
    let contract_info = StoredContractInfo {
        address: deps.api.addr_canonicalize(&contract_addr.as_str())?,
        code_hash: contract_code_hash.clone(),
    };
    let project_id = add_project(deps.storage, contract_info)?;

    let status = Success;
    let msg = format!("Registered contract {}", contract_addr);

    let mut resp: Response = Response::default();
    resp.data = Some(to_binary(&ExecuteAnswer::Register {
        status,
        msg,
        project_id,
        project_address: contract_addr,
        project_code_hash: contract_code_hash,
    })?);
    Ok(resp)
}

fn try_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Option<Addr>,
    project_contract_code_id: Option<u64>,
    project_contract_code_hash: Option<String>,
) -> StdResult<Response> {
    let status;
    let msg;

    let sender_address_raw = deps.api.addr_canonicalize(&info.sender.as_str())?;
    let mut config = get_config(deps.storage)?;

    if sender_address_raw != config.owner {
        return Err(StdError::generic_err(format!("Unauthorized")));
    }

    if owner.is_some() {
        config.owner = deps.api.addr_canonicalize(owner.unwrap().as_str())?;
    }

    if project_contract_code_id.is_some() {
        config.project_contract_code_id = project_contract_code_id.unwrap();
    }

    if project_contract_code_hash.is_some() {
        config.project_contract_code_hash = project_contract_code_hash.unwrap().as_bytes().to_vec();
    }

    set_config(
        deps.storage,
        config.owner.clone(),
        config.project_contract_code_id.clone(),
        config.project_contract_code_hash.clone(),
        config.contract_address,
    )?;

    status = Success;
    msg = format!(
        "New config: owner {}, project code id {}, project code hash {}", 
        config.owner,
        config.project_contract_code_id,
        String::from_utf8(config.project_contract_code_hash).unwrap_or_default(),
    );

    let mut resp: Response = Response::default();
    resp.data = Some(to_binary(&ExecuteAnswer::Config { status, msg })?);
    Ok(resp)
}

fn revoke_permit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    permit_name: String,
) -> StdResult<Response> {
    RevokedPermits::revoke_permit(
        deps.storage,
        PREFIX_REVOKED_PERMITS,
        info.sender.as_str(),
        &permit_name,
    );

    let mut resp: Response = Response::default();
    resp.data = Some(to_binary(&ExecuteAnswer::RevokePermit { status: Success })?);
    Ok(resp)
}

#[entry_point]
pub fn query(
    deps: Deps, 
    _env: Env, 
    msg: QueryMsg
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Projects { page, page_size } => query_projects(deps, page, page_size,),
        QueryMsg::ValidatePermit { permit, } => query_validate_permit(deps, permit,),
    }
}

fn query_projects(
    deps: Deps,
    page: u32,
    page_size: u32,
) -> StdResult<Binary> {
    let mut count = 0_u32;
    if page_size < 1 {
        return Err(StdError::generic_err("Invalid page_size"));
    }
    let mut projects: Vec<ContractInfo> = vec![];
    let projects_wrapped = get_projects(deps.storage, page, page_size);
    if projects_wrapped.is_ok() {
        projects = projects_wrapped
            .unwrap()
            .iter()
            .map(|project| project.to_humanized(deps.api).unwrap())
            .collect();
        count = project_count(deps.storage)?;
    }
    to_binary(&QueryAnswer::Projects { projects, count })
}

fn query_validate_permit(
    deps: Deps,
    permit: Permit, 
) -> StdResult<Binary> {
    let config = get_config(deps.storage)?;
    let address = validate(
        deps, 
        PREFIX_REVOKED_PERMITS, 
        &permit, 
        deps.api.addr_humanize(&config.contract_address)?.into_string(),
        None,
    )?;

    to_binary(&QueryAnswer::ValidatePermit {
        address: Addr::unchecked(address),
    })
}
