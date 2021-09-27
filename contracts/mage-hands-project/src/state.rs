use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::type_name;
use cosmwasm_std::{Uint128, CanonicalAddr, Storage, StdResult, StdError, ReadonlyStorage};
use cosmwasm_storage::{ PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use crate::viewing_key::ViewingKey;

pub const FUNDRAISING: u8 = 1_u8;
pub const EXPIRED: u8 = 2_u8;
pub const SUCCESSFUL: u8 = 3_u8;

pub static STATUS_KEY: &[u8] = b"stat";
pub static CREATOR_KEY: &[u8] = b"crea";
pub static TITLE_KEY: &[u8] = b"titl";
pub static DESCRIPTION_KEY: &[u8] = b"desc";
pub static PLEDGED_MESSAGE_KEY: &[u8] = b"plms";
pub static FUNDED_MESSAGE_KEY: &[u8] = b"fnms";
pub static GOAL_KEY: &[u8] = b"goal";
pub static DEADLINE_KEY: &[u8] = b"dead";
pub static TOTAL_KEY: &[u8] = b"totl";
pub static FEE_KEY: &[u8] = b"feek";
pub static UPFRONT_KEY: &[u8] = b"upfr";
pub static COMMISSION_ADDR_KEY: &[u8] = b"comm";

pub static FUNDER_LIST_PREFIX: &[u8] = b"fund";
pub static FUNDER_AMOUNT_PREFIX: &[u8] = b"amnt";

pub const PREFIX_VIEWING_KEY: &[u8] = b"vkey";
pub const SEED_KEY: &[u8] = b"seed";

pub fn set_prng_seed<S: Storage>(storage: &mut S, prng_seed: &Vec<u8>) -> StdResult<()> {
    set_bin_data(storage, SEED_KEY, &prng_seed)
}

pub fn get_prng_seed<S: ReadonlyStorage>(storage: &S) -> StdResult<Vec<u8>> {
    get_bin_data(storage, SEED_KEY)
}

pub fn set_status<S: Storage>(storage: &mut S, new_status: u8) -> StdResult<()> {
    if new_status < 1 || new_status > 3 {
        return Err(StdError::generic_err("Invalid project status"));
    }
    set_bin_data(storage, STATUS_KEY, &new_status)
}

pub fn get_status<S: ReadonlyStorage>(storage: &S) -> StdResult<u8> {
    get_bin_data(storage, STATUS_KEY)
}

pub fn set_creator<S: Storage>(storage: &mut S, creator: &CanonicalAddr) -> StdResult<()> {
    set_bin_data(storage, CREATOR_KEY, &creator)
}

pub fn get_creator<S: ReadonlyStorage>(storage: &S) -> StdResult<CanonicalAddr> {
    get_bin_data(storage, CREATOR_KEY)
}

pub fn set_title<S: Storage>(storage: &mut S, title: String) -> StdResult<()> {
    set_bin_data(storage, TITLE_KEY, &title.as_bytes().to_vec())
}

pub fn get_title<S: ReadonlyStorage>(storage: &S) -> String {
    let stored_title: Vec<u8> = match get_bin_data(storage, TITLE_KEY) {
        Ok(title) => title,
        Err(_) => vec![],
    };
    String::from_utf8(stored_title).ok().unwrap_or_default()
}

pub fn set_description<S: Storage>(storage: &mut S, description: String) -> StdResult<()> {
    set_bin_data(storage, DESCRIPTION_KEY, &description.as_bytes().to_vec())
}

pub fn get_description<S: ReadonlyStorage>(storage: &S) -> String {
    let stored_description: Vec<u8> = match get_bin_data(storage, DESCRIPTION_KEY) {
        Ok(description) => description,
        Err(_) => vec![],
    };
    String::from_utf8(stored_description).ok().unwrap_or_default()
}

pub fn set_pledged_message<S: Storage>(storage: &mut S, pledged_message: String) -> StdResult<()> {
    set_bin_data(storage, PLEDGED_MESSAGE_KEY, &pledged_message.as_bytes().to_vec())
}

pub fn get_pledged_message<S: ReadonlyStorage>(storage: &S) -> String {
    let stored_pledged_message: Vec<u8> = match get_bin_data(storage, PLEDGED_MESSAGE_KEY) {
        Ok(pledged_message) => pledged_message,
        Err(_) => vec![],
    };
    String::from_utf8(stored_pledged_message).ok().unwrap_or_default()
}

pub fn set_funded_message<S: Storage>(storage: &mut S, funded_message: String) -> StdResult<()> {
    set_bin_data(storage, FUNDED_MESSAGE_KEY, &funded_message.as_bytes().to_vec())
}

pub fn get_funded_message<S: ReadonlyStorage>(storage: &S) -> String {
    let stored_funded_message: Vec<u8> = match get_bin_data(storage, FUNDED_MESSAGE_KEY) {
        Ok(funded_message) => funded_message,
        Err(_) => vec![],
    };
    String::from_utf8(stored_funded_message).ok().unwrap_or_default()
}

pub fn set_goal<S: Storage>(storage: &mut S, goal: u128) -> StdResult<()> {
    set_bin_data(storage, GOAL_KEY, &goal)
}

pub fn get_goal<S: ReadonlyStorage>(storage: &S) -> StdResult<u128> {
    get_bin_data(storage, GOAL_KEY)
}

pub fn set_deadline<S: Storage>(storage: &mut S, deadline: u64) -> StdResult<()> {
    set_bin_data(storage, DEADLINE_KEY, &deadline)
}

pub fn get_deadline<S: ReadonlyStorage>(storage: &S) -> StdResult<u64> {
    get_bin_data(storage, DEADLINE_KEY)
}

pub fn set_total<S: Storage>(storage: &mut S, total: u128) -> StdResult<()> {
    set_bin_data(storage, TOTAL_KEY, &total)
}

pub fn get_total<S: ReadonlyStorage>(storage: &S) -> StdResult<u128> {
    get_bin_data(storage, TOTAL_KEY)
}

pub fn set_fee<S: Storage>(storage: &mut S, fee: StoredFee) -> StdResult<()> {
    set_bin_data(storage, FEE_KEY, &fee)
}

pub fn get_fee<S: ReadonlyStorage>(storage: &S) -> StdResult<StoredFee> {
    get_bin_data(storage, FEE_KEY)
}

pub fn set_upfront<S: Storage>(storage: &mut S, upfront: u128) -> StdResult<()> {
    set_bin_data(storage, UPFRONT_KEY, &upfront)
}

pub fn get_upfront<S: ReadonlyStorage>(storage: &S) -> StdResult<u128> {
    get_bin_data(storage, UPFRONT_KEY)
}

pub fn set_commission_addr<S: Storage>(storage: &mut S, commission_addr: &CanonicalAddr) -> StdResult<()> {
    set_bin_data(storage, COMMISSION_ADDR_KEY, &commission_addr)
}

pub fn get_commission_addr<S: ReadonlyStorage>(storage: &S) -> StdResult<CanonicalAddr> {
    get_bin_data(storage, COMMISSION_ADDR_KEY)
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredFunder {
    pub idx: u32,
    // stay anonymous to project owner
    pub anonymous: bool,
    pub amount: u128,
}

pub fn set_funder<S: Storage>(
    storage: &mut S, 
    funder_addr: &CanonicalAddr, 
    idx: u32, 
    anonymous: bool, 
    amount: u128
) -> StdResult<()> {
    set_bin_data(storage, funder_addr.as_slice(), &StoredFunder{
        idx,
        anonymous,
        amount,
    })
}

pub fn get_funder<S: ReadonlyStorage>(storage: &S, funder_addr: &CanonicalAddr) -> StdResult<StoredFunder> {
    get_bin_data(storage, funder_addr.as_slice())
}

pub fn push_funder<S: Storage>(storage: &mut S, funder_addr: &CanonicalAddr) -> StdResult<u32> {
    let mut store = PrefixedStorage::new(&FUNDER_LIST_PREFIX, storage);
    let mut store = AppendStoreMut::<CanonicalAddr, _>::attach_or_create(&mut store)?;
    store.push(&funder_addr)?;
    Ok(store.len() - 1)
}

pub fn add_funds<S: Storage>(
    storage: &mut S, 
    funder_addr: &CanonicalAddr, 
    anonymous: bool, 
    amount: u128
) -> StdResult<()> {
    // check if has previously put funds in
    let stored_funder = get_funder(storage, funder_addr);
    match stored_funder {
        Ok(stored_funder) => {
            set_funder(storage, funder_addr, stored_funder.idx, anonymous, stored_funder.amount + amount)?;
        },
        Err(_) => {
            let idx = push_funder(storage, funder_addr)?;
            set_funder(storage, funder_addr, idx, anonymous, amount)?;
        }
    };
    let prev_total = get_total(storage)?;
    set_total(storage, prev_total + amount)?;
    Ok(())
}

pub fn clear_funds<S: Storage>(storage: &mut S, funder_addr: &CanonicalAddr) -> StdResult<u128> {
    let stored_funder = get_funder(storage, funder_addr)?;
    if stored_funder.amount > 0 {
        let prev_total = get_total(storage)?;
        set_total(storage, prev_total - stored_funder.amount)?;
    }
    set_funder(storage, funder_addr, stored_funder.idx, true, 0_u128)?;
    Ok(stored_funder.amount)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Funder {
    pub address: Option<CanonicalAddr>,
    pub amount: u128,
}

pub fn get_funders<S: ReadonlyStorage>(
    storage: &S,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<Funder>> {
    let store =
        ReadonlyPrefixedStorage::new(&FUNDER_LIST_PREFIX, storage);

    let store = if let Some(result) = AppendStore::<CanonicalAddr, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    let funder_iter = store
        .iter()
        .skip((page * page_size) as _)
        .take(page_size as _);
    let funders: StdResult<Vec<Funder>> = funder_iter
        .map(|funder| funder.map(|funder| {
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
                },
                Err(_) => {
                    Funder {
                        address: None,
                        amount: 0_u128,
                    }
                }
            }
        }))
        .collect();
    funders
}

//
// Fee
//

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Fee {
    pub commission_rate_nom: Uint128,
    pub commission_rate_denom: Uint128,
}

impl Fee {
    pub fn into_stored(self) -> StdResult<StoredFee> {
        let fee = StoredFee {
            commission_rate_nom: self.commission_rate_nom.u128(),
            commission_rate_denom: self.commission_rate_denom.u128(),
        };
        Ok(fee)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StoredFee {
    pub commission_rate_nom: u128,
    pub commission_rate_denom: u128,
}

impl StoredFee {
    pub fn into_humanized(self) -> StdResult<Fee> {
        let fee = Fee {
            commission_rate_nom: Uint128(self.commission_rate_nom),
            commission_rate_denom: Uint128(self.commission_rate_denom),
        };
        Ok(fee)
    }
}

//
// Viewing Keys
//

pub fn write_viewing_key<S: Storage>(store: &mut S, owner: &CanonicalAddr, key: &ViewingKey) {
    let mut user_key_store = PrefixedStorage::new(PREFIX_VIEWING_KEY, store);
    user_key_store.set(owner.as_slice(), &key.to_hashed());
}

pub fn read_viewing_key<S: Storage>(store: &S, owner: &CanonicalAddr) -> Option<Vec<u8>> {
    let user_key_store = ReadonlyPrefixedStorage::new(PREFIX_VIEWING_KEY, store);
    user_key_store.get(owner.as_slice())
}

//
// Bin data storage setters and getters
//

pub fn set_bin_data<T: Serialize, S: Storage>(
    storage: &mut S,
    key: &[u8],
    data: &T,
) -> StdResult<()> {
    let bin_data =
        bincode2::serialize(&data).map_err(|e| StdError::serialize_err(type_name::<T>(), e))?;
    storage.set(key, &bin_data);
    Ok(())
}

pub fn get_bin_data<T: DeserializeOwned, S: ReadonlyStorage>(
    storage: &S,
    key: &[u8],
) -> StdResult<T> {
    let bin_data = storage.get(key);
    match bin_data {
        None => Err(StdError::not_found("Key not found in storage")),
        Some(bin_data) => Ok(bincode2::deserialize::<T>(&bin_data)
            .map_err(|e| StdError::serialize_err(type_name::<T>(), e))?),
    }
}