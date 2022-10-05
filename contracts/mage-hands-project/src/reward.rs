use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint128, CanonicalAddr, Binary, };

pub const LINEAR_WEIGHT: u8 = 1;
pub const SQRT_WEIGHT: u8 = 2;
pub const LOG_WEIGHT: u8 = 3;

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

    // total number of tokens to hold in vesting contract
    pub amount: Uint128,

    // timeline of release of tokens to contributors
    // sum of per_mille in each VestingEvent must sum to 1000
    pub contributors_vesting_schedule: Vec<VestingEvent>,
    // permille of tokens that are reserved for contributors
    pub contributors_per_mille: u16,
    // minimum contribution to be eligible for reward
    pub minimum_contribution: Uint128,
    // maximum amount of contribution applied to reward
    // if maximum == 0 or < minimum, then there is no maximum
    pub maximum_contribution: Uint128,
    // contribution weighting: one of linear, sqrt, or log
    // weighting is applied after minimum and maximum is applied
    pub contribution_weight: u8,

    // timeline of release of tokens to creator
    // sum of per_mille in each VestingEvent must sum to 1000
    pub creator_vesting_schedule: Vec<VestingEvent>,
    // permille of tokens that are reserved for creator
    pub creator_per_mille: u16,
    // addresses to evenly distribute coins to
    // if none, will distribute to project creator address
    pub creator_addresses: Option<Vec<Addr>>,
}

// Vesting events indicate what block a share of the reward becomes valid
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug,)]
pub struct VestingEvent {
    pub block: u64,
    pub per_mille: u16,
}

// Vesting reward indicate how much the reward is at each block for a given contributor
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug,)]
pub struct VestingReward {
    pub block: u64,
    pub amount: u128,
}

// Status of vesting rewards sent in status message
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug,)]
pub struct VestingRewardStatus {
    pub amount: Uint128,
    pub block: u64,
    pub received: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug,)]
pub struct StoredSnip24RewardInit {
    pub reward_snip24_code_id: u64,
    pub reward_snip24_code_hash: String,

    // snip24 params
    pub name: String,
    pub admin: Option<CanonicalAddr>,
    pub symbol: String,
    pub decimals: u8,
    pub public_total_supply: bool,
    pub enable_deposit: bool,
    pub enable_redeem: bool,
    pub enable_mint: bool,
    pub enable_burn: bool,

    // total number of tokens to hold in vesting contract
    pub amount: u128,

    // timeline of release of tokens to contributors
    // sum of per_mille in each VestingEvent must sum to 1000
    pub contributors_vesting_schedule: Vec<VestingEvent>,
    // permille of tokens that are reserved for contributors
    pub contributors_per_mille: u16,
    // minimum contribution to be eligible for reward
    pub minimum_contribution: u128,
    // maximum amount of contribution applied to reward
    // if maximum == 0 or < minimum, then there is no maximum
    pub maximum_contribution: u128,
    // contribution weighting: one of linear, sqrt, or log
    // weighting is applied after minimum and maximum is applied
    pub contribution_weight: u8,

    // timeline of release of tokens to creator
    // sum of per_mille in each VestingEvent must sum to 1000
    pub creator_vesting_schedule: Vec<VestingEvent>,
    // permille of tokens that are reserved for creator
    pub creator_per_mille: u16,
    // addresses to evenly distribute coins to
    // if none, will distribute to project creator address
    pub creator_addresses: Option<Vec<CanonicalAddr>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug,)]
pub struct RewardMessage {
    pub threshold: Uint128,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug,)]
pub struct StoredRewardMessage {
    pub threshold: u128,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug,)]
pub struct Snip24InstantiateMsg {
    pub name: String,
    pub admin: Option<Addr>,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Option<Vec<InitialBalance>>,
    pub prng_seed: Binary,
    pub config: Option<InitConfig>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct InitialBalance {
    pub address: Addr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "snake_case")]
pub struct InitConfig {
    pub public_total_supply: Option<bool>,
    pub enable_deposit: Option<bool>,
    pub enable_redeem: Option<bool>,
    pub enable_mint: Option<bool>,
    pub enable_burn: Option<bool>,
}