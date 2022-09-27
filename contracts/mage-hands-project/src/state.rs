use crate::viewing_key::ViewingKey;
use cosmwasm_std::{CanonicalAddr, StdError, StdResult, Storage,};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::type_name;

pub const FUNDRAISING: u8 = 1_u8;
pub const EXPIRED: u8 = 2_u8;
pub const SUCCESSFUL: u8 = 3_u8;

pub static CONFIG_KEY: &[u8] = b"conf";
pub static STATUS_KEY: &[u8] = b"stat";
pub static CREATOR_KEY: &[u8] = b"crea";
pub static TITLE_KEY: &[u8] = b"titl";
pub static SUBTITLE_KEY: &[u8] = b"subt";
pub static DESCRIPTION_KEY: &[u8] = b"desc";
pub static PLEDGED_MESSAGE_KEY: &[u8] = b"plms";
pub static FUNDED_MESSAGE_KEY: &[u8] = b"fnms";
pub static GOAL_KEY: &[u8] = b"goal";
pub static DEADLINE_KEY: &[u8] = b"dead";
pub static DEADMAN_KEY: &[u8] = b"dman";
pub static CATEGORIES_KEY: &[u8] = b"cate";
pub static TOTAL_KEY: &[u8] = b"totl";

pub static FUNDER_STORE: AppendStore<CanonicalAddr> = AppendStore::new(b"fund");
pub static COMMENT_STORE: AppendStore<String> = AppendStore::new(b"comm");

pub const PREFIX_VIEWING_KEY: &[u8] = b"vkey";
pub const SEED_KEY: &[u8] = b"seed";

pub static PAID_OUT_KEY: &[u8] = b"pout";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub platform_contract: CanonicalAddr,
    pub platform_hash: String,
    pub snip20_contract: CanonicalAddr,
    pub snip20_hash: String,
}

pub fn set_config(
    storage: &mut dyn Storage,
    platform_contract: CanonicalAddr,
    platform_hash: String,
    snip20_contract: CanonicalAddr,
    snip20_hash: String,
) -> StdResult<()> {
    let config = Config {
        platform_contract,
        platform_hash,
        snip20_contract,
        snip20_hash,
    };
    set_bin_data(storage, CONFIG_KEY, &config)
}

pub fn get_config(storage: &dyn Storage) -> StdResult<Config> {
    get_bin_data(storage, CONFIG_KEY)
}

pub fn set_prng_seed(storage: &mut dyn Storage, prng_seed: &Vec<u8>) -> StdResult<()> {
    set_bin_data(storage, SEED_KEY, &prng_seed)
}

pub fn get_prng_seed(storage: &dyn Storage) -> StdResult<Vec<u8>> {
    get_bin_data(storage, SEED_KEY)
}

pub fn set_status(storage: &mut dyn Storage, new_status: u8) -> StdResult<()> {
    if new_status < 1 || new_status > 3 {
        return Err(StdError::generic_err("Invalid project status"));
    }
    set_bin_data(storage, STATUS_KEY, &new_status)
}

pub fn get_status(storage: &dyn Storage) -> StdResult<u8> {
    get_bin_data(storage, STATUS_KEY)
}

pub fn set_creator(storage: &mut dyn Storage, creator: &CanonicalAddr) -> StdResult<()> {
    set_bin_data(storage, CREATOR_KEY, &creator)
}

pub fn get_creator(storage: &dyn Storage) -> StdResult<CanonicalAddr> {
    get_bin_data(storage, CREATOR_KEY)
}

pub fn set_title(storage: &mut dyn Storage, title: String) -> StdResult<()> {
    set_bin_data(storage, TITLE_KEY, &title.as_bytes().to_vec())
}

pub fn get_title(storage: &dyn Storage) -> String {
    let stored_title: Vec<u8> = match get_bin_data(storage, TITLE_KEY) {
        Ok(title) => title,
        Err(_) => vec![],
    };
    String::from_utf8(stored_title).ok().unwrap_or_default()
}

pub fn set_subtitle(storage: &mut dyn Storage, subtitle: String) -> StdResult<()> {
    set_bin_data(storage, SUBTITLE_KEY, &subtitle.as_bytes().to_vec())
}

pub fn get_subtitle(storage: &dyn Storage) -> String {
    let stored_subtitle: Vec<u8> = match get_bin_data(storage, SUBTITLE_KEY) {
        Ok(subtitle) => subtitle,
        Err(_) => vec![],
    };
    String::from_utf8(stored_subtitle).ok().unwrap_or_default()
}

pub fn set_description(storage: &mut dyn Storage, description: String) -> StdResult<()> {
    let bytes = base64::decode(&description).map_err(StdError::invalid_base64)?;
    set_bin_data(storage, DESCRIPTION_KEY, &bytes)
}

pub fn get_description(storage: &dyn Storage) -> String {
    let stored_description: Vec<u8> = match get_bin_data(storage, DESCRIPTION_KEY) {
        Ok(description) => description,
        Err(_) => vec![],
    };
    base64::encode(stored_description)
}

pub fn set_pledged_message(storage: &mut dyn Storage, pledged_message: String) -> StdResult<()> {
    let bytes = base64::decode(&pledged_message).map_err(StdError::invalid_base64)?;
    set_bin_data(
        storage,
        PLEDGED_MESSAGE_KEY,
        &bytes,
    )
}

pub fn get_pledged_message(storage: &dyn Storage) -> String {
    let stored_pledged_message: Vec<u8> = match get_bin_data(storage, PLEDGED_MESSAGE_KEY) {
        Ok(pledged_message) => pledged_message,
        Err(_) => vec![],
    };
    base64::encode(stored_pledged_message)
}

pub fn set_funded_message(storage: &mut dyn Storage, funded_message: String) -> StdResult<()> {
    let bytes = base64::decode(&funded_message).map_err(StdError::invalid_base64)?;
    set_bin_data(
        storage,
        FUNDED_MESSAGE_KEY,
        &bytes,
    )
}

pub fn get_funded_message(storage: &dyn Storage) -> String {
    let stored_funded_message: Vec<u8> = match get_bin_data(storage, FUNDED_MESSAGE_KEY) {
        Ok(funded_message) => funded_message,
        Err(_) => vec![],
    };
    base64::encode(stored_funded_message)
}

pub fn set_goal(storage: &mut dyn Storage, goal: u128) -> StdResult<()> {
    set_bin_data(storage, GOAL_KEY, &goal)
}

pub fn get_goal(storage: &dyn Storage) -> StdResult<u128> {
    get_bin_data(storage, GOAL_KEY)
}

pub fn set_deadline(storage: &mut dyn Storage, deadline: u64) -> StdResult<()> {
    set_bin_data(storage, DEADLINE_KEY, &deadline)
}

pub fn get_deadline(storage: &dyn Storage) -> StdResult<u64> {
    get_bin_data(storage, DEADLINE_KEY)
}

pub fn set_deadman(storage: &mut dyn Storage, deadman: u64) -> StdResult<()> {
    set_bin_data(storage, DEADMAN_KEY, &deadman)
}

pub fn get_deadman(storage: &dyn Storage) -> StdResult<u64> {
    get_bin_data(storage, DEADMAN_KEY)
}

pub fn set_categories(storage: &mut dyn Storage, categories: Vec<u16>) -> StdResult<()> {
    set_bin_data(storage, CATEGORIES_KEY, &categories)
}

pub fn get_categories(storage: &dyn Storage) -> StdResult<Vec<u16>> {
    get_bin_data(storage, CATEGORIES_KEY)
}

pub fn set_total(storage: &mut dyn Storage, total: u128) -> StdResult<()> {
    set_bin_data(storage, TOTAL_KEY, &total)
}

pub fn get_total(storage: &dyn Storage) -> StdResult<u128> {
    get_bin_data(storage, TOTAL_KEY)
}

pub fn push_comment(storage: &mut dyn Storage, comment: String) -> StdResult<u32> {
    COMMENT_STORE.push(storage, &comment)?;
    Ok(COMMENT_STORE.get_len(storage)? - 1)
}

pub fn get_comments(
    storage: &dyn Storage,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<String>> {
    let comments: StdResult<Vec<String>> = COMMENT_STORE
        .iter(storage)?
        .skip((page * page_size) as _)
        .take(page_size as _)
        .collect();
    comments
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredFunder {
    pub idx: u32,
    // stay anonymous to project owner
    pub anonymous: bool,
    pub amount: u128,
}

pub fn set_funder(
    storage: &mut dyn Storage,
    funder_addr: &CanonicalAddr,
    idx: u32,
    anonymous: bool,
    amount: u128,
) -> StdResult<()> {
    set_bin_data(
        storage,
        funder_addr.as_slice(),
        &StoredFunder {
            idx,
            anonymous,
            amount,
        },
    )
}

pub fn get_funder(
    storage: &dyn Storage,
    funder_addr: &CanonicalAddr,
) -> StdResult<StoredFunder> {
    get_bin_data(storage, funder_addr.as_slice())
}

pub fn push_funder(storage: &mut dyn Storage, funder_addr: &CanonicalAddr) -> StdResult<u32> {
    FUNDER_STORE.push(storage, &funder_addr)?;
    Ok(FUNDER_STORE.get_len(storage)? - 1)
}

pub fn add_funds(
    storage: &mut dyn Storage,
    funder_addr: &CanonicalAddr,
    anonymous: bool,
    amount: u128,
) -> StdResult<()> {
    // check if has previously put funds in
    let stored_funder = get_funder(storage, funder_addr);
    match stored_funder {
        Ok(stored_funder) => {
            set_funder(
                storage,
                funder_addr,
                stored_funder.idx,
                anonymous,
                stored_funder.amount + amount,
            )?;
        }
        Err(_) => {
            let idx = push_funder(storage, funder_addr)?;
            set_funder(storage, funder_addr, idx, anonymous, amount)?;
        }
    };
    let prev_total = get_total(storage)?;
    set_total(storage, prev_total + amount)?;
    Ok(())
}

pub fn clear_funds(storage: &mut dyn Storage, funder_addr: &CanonicalAddr) -> StdResult<u128> {
    let stored_funder = get_funder(storage, funder_addr)?;
    if stored_funder.amount > 0 {
        let prev_total = get_total(storage)?;
        set_total(storage, prev_total - stored_funder.amount)?;
    }
    set_funder(storage, funder_addr, stored_funder.idx, true, 0_u128)?;
    Ok(stored_funder.amount)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Funder {
    pub address: Option<CanonicalAddr>,
    pub amount: u128,
}

pub fn get_funders(
    storage: &dyn Storage,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<Funder>> {
    let funder_iter = FUNDER_STORE
        .iter(storage)?
        .skip((page * page_size) as _)
        .take(page_size as _);
    let funders: StdResult<Vec<Funder>> = funder_iter
        .map(|funder| {
            funder.map(|funder| {
                let stored_funder = get_funder(storage, &funder);
                match stored_funder {
                    Ok(stored_funder) => {
                        if stored_funder.anonymous {
                            Funder {
                                address: None,
                                amount: stored_funder.amount,
                            }
                        } else {
                            Funder {
                                address: Some(funder),
                                amount: stored_funder.amount,
                            }
                        }
                    }
                    Err(_) => Funder {
                        address: None,
                        amount: 0_u128,
                    },
                }
            })
        })
        .collect();
    funders
}

pub fn paid_out(storage: &mut dyn Storage) -> StdResult<()> {
    set_bin_data(storage, PAID_OUT_KEY, &true)
}

pub fn is_paid_out(storage: &dyn Storage) -> bool {
    get_bin_data(storage, PAID_OUT_KEY).unwrap_or_else(|_| false)
}

//
// Viewing Keys
//

pub fn write_viewing_key(storage: &mut dyn Storage, owner: &CanonicalAddr, key: &ViewingKey) {
    let mut user_key_store = PrefixedStorage::new(storage, PREFIX_VIEWING_KEY);
    user_key_store.set(owner.as_slice(), &key.to_hashed());
}

pub fn read_viewing_key(storage: &dyn Storage, owner: &CanonicalAddr) -> Option<Vec<u8>> {
    let user_key_store = ReadonlyPrefixedStorage::new(storage, PREFIX_VIEWING_KEY);
    user_key_store.get(owner.as_slice())
}

//
// Bin data storage setters and getters
//

pub fn set_bin_data<T: Serialize>(
    storage: &mut dyn Storage,
    key: &[u8],
    data: &T,
) -> StdResult<()> {
    let bin_data =
        bincode2::serialize(&data).map_err(|e| StdError::serialize_err(type_name::<T>(), e))?;
    storage.set(key, &bin_data);
    Ok(())
}

pub fn get_bin_data<T: DeserializeOwned>(
    storage: &dyn Storage,
    key: &[u8],
) -> StdResult<T> {
    let bin_data = storage.get(key);
    match bin_data {
        None => Err(StdError::not_found("Key not found in storage")),
        Some(bin_data) => Ok(bincode2::deserialize::<T>(&bin_data)
            .map_err(|e| StdError::serialize_err(type_name::<T>(), e))?),
    }
}
