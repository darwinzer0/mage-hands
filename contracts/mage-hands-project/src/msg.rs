use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Binary, HumanAddr, StdResult, Uint128};
use crate::state::Fee;

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
    pub deadline: u64,

    // commission
    pub commission_addr: HumanAddr,
    pub upfront: Uint128,
    pub fee: Fee,

    pub padding: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    ChangeText {
        title: Option<String>,
        description: Option<String>,
        pledged_message: Option<String>,
        funded_message: Option<String>,
        padding: Option<String>,
    },
    // project owner: sets project immediately to EXPIRED
    Cancel {
        padding: Option<String>,
    },
    // project funder: withdraw funds that you have pledged to this project (state must be FUNDRAISING or EXPIRED)
    Refund {
        padding: Option<String>,
    },
    // project owner: withdraw funding (state must be SUCCESSFUL)
    PayOut {
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
    Refund {
        status: ResponseStatus,
        msg: String,
    },
    PayOut {
        status: ResponseStatus,
        msg: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetStatus returns the current status: Fundraising, Expired, or Successful
    GetStatus {},
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    GetStatus {
        status: String,
    }
}
