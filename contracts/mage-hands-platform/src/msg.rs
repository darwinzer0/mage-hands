use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::InitCallback;
use serde::{Deserialize, Serialize};

use crate::state::Fee;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub owner: Option<HumanAddr>,
    pub default_upfront: Uint128,
    pub default_fee: Fee,

    pub project_contract_code_id: u64,
    pub project_contract_code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProjectInitMsg {
    pub creator: HumanAddr,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: String,
    // message for people who have pledged money before funding has been completed
    pub pledged_message: Option<String>,
    // message for people who have contributed money after project is successfully funded
    pub funded_message: Option<String>,
    pub goal: Uint128,
    pub deadline: u64,
    pub categories: Vec<u16>,

    // commission
    pub commission_addr: HumanAddr,
    pub upfront: Uint128,
    pub fee: Fee,

    pub entropy: String,

    pub source_contract: HumanAddr,
    pub source_hash: String,

    pub padding: Option<String>,
}

impl InitCallback for ProjectInitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // create a new project
    Create {
        title: String,
        subtitle: Option<String>,
        description: String,
        pledged_message: Option<String>,
        funded_message: Option<String>,
        goal: Uint128,
        deadline: u64,
        categories: Vec<u16>,
        entropy: String, // used to set up prng in project contract
        padding: Option<String>,
    },
    // owner only
    Config {
        owner: Option<HumanAddr>,
        default_upfront: Option<Uint128>,
        default_fee: Option<Fee>,
        project_contract_code_id: Option<u64>,
        project_contract_code_hash: Option<String>,
        padding: Option<String>,
    },
    // register a project contract
    Register {
        contract_addr: HumanAddr,
        contract_code_hash: String,
    },

    // Permit
    RevokePermit {
        permit_name: String,
        padding: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Create {
        status: ResponseStatus,
        msg: String,
    },
    Config {
        status: ResponseStatus,
        msg: String,
    },
    Register {
        status: ResponseStatus,
        msg: String,
        project_id: u32,
        project_address: HumanAddr,
        project_code_hash: String,
    },
    // Permit
    RevokePermit {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // gets a paginated list of projects
    Projects { page: u32, page_size: u32 },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Projects {
        projects: Vec<ContractInfo>,
        count: u32,
    },
}

/// code hash and address of a contract
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
pub struct ContractInfo {
    /// contract's code hash string
    pub code_hash: String,
    /// contract's address
    pub address: HumanAddr,
}

// Take a Vec<u8> and pad it up to a multiple of `block_size`, using spaces at the end.
pub fn space_pad(block_size: usize, message: &mut Vec<u8>) -> &mut Vec<u8> {
    let len = message.len();
    let surplus = len % block_size;
    if surplus == 0 {
        return message;
    }

    let missing = block_size - surplus;
    message.reserve(missing);
    message.extend(std::iter::repeat(b' ').take(missing));
    message
}