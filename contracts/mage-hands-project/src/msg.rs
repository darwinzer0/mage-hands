use crate::reward::Snip24RewardInit;
use crate::viewing_key::ViewingKey;
use cosmwasm_std::{Addr, Uint128, Binary, };
use secret_toolkit::permit::Permit;
use secret_toolkit::utils::{HandleCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {
    pub creator: Addr,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: String,
    // message for people who have pledged money before funding has been completed
    pub pledged_message: Option<String>,
    // message for people who have contributed money after project is funded
    pub funded_message: Option<String>,
    pub goal: Uint128,
    // deadline (block height)
    pub deadline: u64,
    // deadman expiration for funded project (in blocks)
    pub deadman: u64,
    pub categories: Vec<u16>,

    pub entropy: String,

    pub source_contract: Addr,
    pub source_hash: String,

    // contract info for snip20/4 contributions
    pub snip20_contract: Addr,
    pub snip20_hash: String, 

    // new snip24
    pub snip24_reward_init: Option<Snip24RewardInit>,

    pub padding: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlatformExecuteMsg {
    Register {
        contract_addr: Addr,
        contract_code_hash: String,
    },
}

impl HandleCallback for PlatformExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // project creator: change the title, description, ...
    ChangeText {
        title: Option<String>,
        subtitle: Option<String>,
        description: Option<String>,
        pledged_message: Option<String>,
        funded_message: Option<String>,
        categories: Option<Vec<u16>>,
        padding: Option<String>,
    },
    // Receives from a SNIP-20, project funder: contribute funds to this project
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        msg: Option<Binary>,
    },
    // project funder: withdraw funds that you have pledged to this project (state must be FUNDRAISING or EXPIRED)
    Refund {
        padding: Option<String>,
    },
    // project creator: sets project immediately to EXPIRED
    Cancel {
        padding: Option<String>,
    },
    // project creator: withdraw funding (state must be SUCCESSFUL and deadline past)
    PayOut {
        padding: Option<String>,
    },
    // comment on the project
    Comment {
        comment: String,
        padding: Option<String>,
    },
    // flag project as spam
    FlagSpam {
        flag: bool,
        padding: Option<String>,
    },
    GenerateViewingKey {
        entropy: String,
        padding: Option<String>,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteReceiveMsg {
    ReceiveContribution {
        anon: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteAnswer {
    ChangeText {
        status: ResponseStatus,
        msg: String,
    },
    Cancel {
        status: ResponseStatus,
        msg: String,
    },
    Receive {
        status: ResponseStatus,
        msg: String,
    },
    Refund {
        status: ResponseStatus,
        msg: String,
    },
    PayOut {
        status: ResponseStatus,
        msg: String,
    },
    Comment {
        status: ResponseStatus,
        msg: String,
    },
    FlagSpam {
        spam_count: u32,
        status: ResponseStatus,
        msg: String,
    },
    GenerateViewingKey {
        key: ViewingKey,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetStatus returns the current status: Fundraising, Expired, or Successful
    Status {},
    StatusAuth { address: Addr, key: String },
    StatusWithPermit { permit: Permit },
    Comments { page: u32, page_size: u32 },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (Vec<&Addr>, ViewingKey) {
        match self {
            Self::StatusAuth { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            _ => panic!("This query type does not require authentication"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Status {
        creator: Addr,
        status: String,
        paid_out: bool,
        goal: Uint128,
        total: Uint128,
        deadline: u64,
        deadman: u64,
        title: String,
        subtitle: String,
        description: String,
        categories: Vec<u16>,
        spam_count: u32,
    },
    StatusAuth {
        creator: Addr,
        status: String,
        paid_out: bool,
        goal: Uint128,
        total: Uint128,
        deadline: u64,
        deadman: u64,
        title: String,
        subtitle: String,
        description: String,
        categories: Vec<u16>,
        spam_count: u32,
        pledged_message: Option<String>,
        funded_message: Option<String>,
        contribution: Option<Uint128>,
    },
    StatusWithPermit {
        creator: Addr,
        status: String,
        paid_out: bool,
        goal: Uint128,
        total: Uint128,
        deadline: u64,
        deadman: u64,
        title: String,
        subtitle: String,
        description: String,
        categories: Vec<u16>,
        spam_count: u32,
        pledged_message: Option<String>,
        funded_message: Option<String>,
        contribution: Option<Uint128>,
    },
    Comments {
        comments: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PlatformQueryMsg {
    ValidatePermit {     
        permit: Permit, 
    },
}

impl Query for PlatformQueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ValidatePermitInnerResponse {
    pub address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ValidatePermitResponse {
    pub validate_permit: ValidatePermitInnerResponse,
}