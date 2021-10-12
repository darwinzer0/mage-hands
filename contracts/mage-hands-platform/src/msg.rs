use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128};
use secret_toolkit::utils::{InitCallback,};

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
        project_addr: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // gets a paginated list of projects
    Projects {
        page: u32,
        page_size: u32,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Projects {
        projects: Vec<HumanAddr>,
        count: u32,
    },
}
