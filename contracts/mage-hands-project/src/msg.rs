use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Binary, HumanAddr, StdResult, Uint128};
use crate::state::Fee;
use crate::viewing_key::ViewingKey;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub creator: HumanAddr,
    pub title: String,
    pub description: String,
    // message for people who have pledged money before funding has been completed
    pub pledged_message: Option<String>,
    // message for people who have contributed money after project is funded
    pub funded_message: Option<String>,
    pub goal: Uint128,
    pub deadline: i32,

    // commission
    pub commission_addr: HumanAddr,
    pub upfront: Uint128,
    pub fee: Fee,

    pub entropy: String,

    pub padding: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // project creator: change the title, description, ...
    ChangeText {
        title: Option<String>,
        description: Option<String>,
        pledged_message: Option<String>,
        funded_message: Option<String>,
        padding: Option<String>,
    },
    // project funder: contribute funds to this project
    // returns a viewing key
    Contribute {
        anonymous: Option<bool>,
        entropy: String,
        padding: Option<String>,
    },
    // project funder: withdraw funds that you have pledged to this project (state must be FUNDRAISING or EXPIRED)
    Refund {
        padding: Option<String>,
    },
    // project creator: sets project immediately to EXPIRED
    Cancel {
        padding: Option<String>,
    },
    // project creator: withdraw funding (state must be SUCCESSFUL)
    PayOut {
        padding: Option<String>,
    },
    GenerateViewingKey {
        entropy: String,
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
    ChangeText {
        status: ResponseStatus,
        msg: String,
    },
    Cancel {
        status: ResponseStatus,
        msg: String,
    },
    Contribute {
        status: ResponseStatus,
        msg: String,
        key: Option<ViewingKey>,
    },
    Refund {
        status: ResponseStatus,
        msg: String,
    },
    PayOut {
        status: ResponseStatus,
        msg: String,
    },
    GenerateViewingKey {
        key: ViewingKey,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetStatus returns the current status: Fundraising, Expired, or Successful
    Status {},
    StatusAuth {
        address: HumanAddr,
        key: String,
    },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (Vec<&HumanAddr>, ViewingKey) {
        match self {
            Self::StatusAuth { address, key, .. } => {
                (vec![address], ViewingKey(key.clone()))
            }
            _ => panic!("This query type does not require authentication"),
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Status {
        creator: HumanAddr,
        status: String,
        goal: Uint128,
        total: Uint128,
        deadline: i32,
        title: String,
        description: String,
    },
    StatusAuth {
        creator: HumanAddr,
        status: String,
        goal: Uint128,
        total: Uint128,
        deadline: i32,
        title: String,
        description: String,
        pledged_message: Option<String>,
        funded_message: Option<String>,
        contribution: Option<Uint128>,
    },
}
