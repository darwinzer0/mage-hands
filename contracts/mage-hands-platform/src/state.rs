use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::type_name;
use cosmwasm_std::{Api, Uint128, CanonicalAddr, HumanAddr, Storage, StdResult, StdError, ReadonlyStorage};
use cosmwasm_storage::{ PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use crate::msg::ContractInfo;

pub static CONFIG_KEY: &[u8] = b"conf";
pub static CREATING_PROJECT_FLAG_KEY: &[u8] = b"flag";
pub const CREATING_PROJECT: bool = true;
pub static PROJECTS_PREFIX: &[u8] = b"proj";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub default_upfront: u128,
    pub default_fee: StoredFee,
    pub project_contract_code_id: u64,
    pub project_contract_code_hash: Vec<u8>,
}

pub fn set_config<S: Storage>(
    storage: &mut S, 
    owner: CanonicalAddr, 
    default_upfront: u128, 
    default_fee: StoredFee,
    project_contract_code_id: u64,
    project_contract_code_hash: Vec<u8>,
) -> StdResult<()> {
    let config = Config {
        owner,
        default_upfront,
        default_fee,
        project_contract_code_id,
        project_contract_code_hash,
    };
    set_bin_data(storage, CONFIG_KEY, &config)
}

pub fn get_config<S: ReadonlyStorage>(storage: &S) -> StdResult<Config> {
    get_bin_data(storage, CONFIG_KEY)
}

pub fn set_creating_project<S: Storage>(storage: &mut S, creating_project: bool) -> StdResult<()> {
    set_bin_data(storage, CREATING_PROJECT_FLAG_KEY, &creating_project)
}

pub fn is_creating_project<S: ReadonlyStorage>(storage: &S) -> bool {
    get_bin_data(storage, CREATING_PROJECT_FLAG_KEY).unwrap_or_else(|_| false)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProjectContract {
    pub address: HumanAddr,
    pub contract_hash: String,
}

/// code hash and address of a contract
#[derive(Serialize, Deserialize, Clone)]
pub struct StoredContractInfo {
    /// contract's code hash string
    pub code_hash: String,
    /// contract's address
    pub address: CanonicalAddr,
}

impl StoredContractInfo {
    /// Returns StdResult<ContractInfo> from converting a StoreContractInfo to a displayable
    /// ContractInfo
    ///
    /// # Arguments
    ///
    /// * `api` - a reference to the Api used to convert human and canonical addresses
    pub fn to_humanized<A: Api>(&self, api: &A) -> StdResult<ContractInfo> {
        let info = ContractInfo {
            address: api.human_address(&self.address)?,
            code_hash: self.code_hash.clone(),
        };
        Ok(info)
    }
}

pub fn project_count<S: ReadonlyStorage>(
    storage: &S,
) -> StdResult<u32> {
    let store = ReadonlyPrefixedStorage::new(&PROJECTS_PREFIX, storage);

    // Try to access the storage of contract addresses and get length
    // If it doesn't exist yet, return 0.
    if let Some(result) = AppendStore::<StoredContractInfo, _>::attach(&store) {
        return Ok(result?.len());
    } else {
        return Ok(0);
    };
}

pub fn add_project<S: Storage>(
    storage: &mut S,
    project: StoredContractInfo,
) -> StdResult<u32> {
    let mut store = PrefixedStorage::new(&PROJECTS_PREFIX, storage);
    let mut store = AppendStoreMut::<StoredContractInfo, _>::attach_or_create(&mut store)?;
    store.push(&project)?;
    Ok(store.len() - 1)
}

pub fn get_projects<S: ReadonlyStorage>(
    storage: &S,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<StoredContractInfo>> {
    let store = ReadonlyPrefixedStorage::new(&PROJECTS_PREFIX, storage);

    // Try to access the storage of contract addresses.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<StoredContractInfo, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `page_size` projects starting from the latest project, potentially skipping `page * page_size`
    // projects from the start.
    let projects = store
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _)
        .collect();
    projects
}

pub fn get_projects_count<S: ReadonlyStorage>(
    storage: &S,
) -> StdResult<u32> {
    let store = ReadonlyPrefixedStorage::new(&PROJECTS_PREFIX, storage);

    // Try to access the storage of contract addresses and return length.
    // If it doesn't exist yet, return 0.
    if let Some(result) = AppendStore::<StoredContractInfo, _>::attach(&store) {
        return Ok(result?.len());
    } else {
        return Ok(0_u32);
    };
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