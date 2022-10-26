
use cosmwasm_std::{Addr, Uint128};
use secret_toolkit::utils::InitCallback;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProjectInstantiateMsg {
    pub creator: Addr,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: String,
    pub cover_img: String,
    // message for people who have pledged money before funding has been completed
    pub pledged_message: Option<String>,
    // message for people who have contributed money after project is successfully funded
    pub funded_message: Option<String>,
    // reward messages based on contribution thresholds
    pub reward_messages: Vec<RewardMessage>,
    pub goal: Uint128,
    pub deadline: u64,
    pub deadman: u64,
    pub categories: Vec<u16>,

    pub entropy: String,

    pub source_contract: Addr,
    pub source_hash: String,

    pub snip20_contract: Addr,
    pub snip20_hash: String,
    // minimum and maximum pledge amounts
    pub minimum_pledge: Uint128,
    pub maximum_pledge: Uint128,

    // new snip24
    pub snip24_reward_init: Option<Snip24RewardInit>,

    pub padding: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug,)]
pub struct RewardMessage {
    pub threshold: Uint128,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug,)]
pub struct Snip24RewardInit {
    pub reward_snip24_code_id: u64,
    pub reward_snip24_code_hash: String,

    // snip24 params
    pub name: String,
    pub admin: Option<Addr>,
    pub symbol: String,
    pub decimals: u8,
    pub public_total_supply: bool,
    pub enable_deposit: bool,
    pub enable_redeem: bool,
    pub enable_mint: bool,
    pub enable_burn: bool,

    // timeline of release of tokens to contributors
    pub contributor_vesting_schedule: Vec<VestingEvent>,
    // minimum contribution to be eligible for reward
    pub minimum_contribution: Option<Uint128>,
    // maximum amount of contribution applied to reward
    // if maximum == 0 or < minimum, then there is no maximum
    pub maximum_contribution: Option<Uint128>,
    // contribution weighting: one of linear, sqrt, or log
    // weighting is applied after minimum and maximum is applied
    pub contribution_weight: u8,

    // timeline of release of tokens to creator
    pub creator_vesting_schedule: Vec<VestingEvent>,
    // addresses to evenly distribute coins to
    // if none, will distribute to project creator address
    pub creator_addresses: Option<Vec<Addr>>,
}

// Vesting events indicate what block
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug,)]
pub struct VestingEvent {
    pub block: u64,
    pub amount: Uint128,
}

impl InitCallback for ProjectInstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}