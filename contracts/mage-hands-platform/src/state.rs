use crate::msg::ContractInfo;
use cosmwasm_std::{
    Api, CanonicalAddr, Addr, StdError, StdResult, Storage,
};
use secret_toolkit::storage::{AppendStore};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::type_name;

pub static CONFIG_KEY: &[u8] = b"conf";
pub static CREATING_PROJECT_FLAG_KEY: &[u8] = b"flag";
pub const CREATING_PROJECT: bool = true;
pub static PROJECTS_STORE: AppendStore<StoredContractInfo> = AppendStore::new(b"proj");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub project_contract_code_id: u64,
    pub project_contract_code_hash: Vec<u8>,
    pub contract_address: CanonicalAddr,
    pub token_min_max_pledges: Vec<StoredPledgeMinMax>,
    pub deadman: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StoredPledgeMinMax {
    pub token_addr: CanonicalAddr,
    pub min: u128,
    pub max: u128,
}

pub fn set_config(
    storage: &mut dyn Storage,
    owner: CanonicalAddr,
    project_contract_code_id: u64,
    project_contract_code_hash: Vec<u8>,
    contract_address: CanonicalAddr,
    token_min_max_pledges: Vec<StoredPledgeMinMax>,
    deadman: u64,
) -> StdResult<()> {
    let config = Config {
        owner,
        project_contract_code_id,
        project_contract_code_hash,
        contract_address,
        token_min_max_pledges,
        deadman,
    };
    set_bin_data(storage, CONFIG_KEY, &config)
}

pub fn get_config(storage: &dyn Storage) -> StdResult<Config> {
    get_bin_data(storage, CONFIG_KEY)
}

pub fn set_creating_project(storage: &mut dyn Storage, creating_project: bool) -> StdResult<()> {
    set_bin_data(storage, CREATING_PROJECT_FLAG_KEY, &creating_project)
}

pub fn is_creating_project(storage: &dyn Storage) -> bool {
    get_bin_data(storage, CREATING_PROJECT_FLAG_KEY).unwrap_or_else(|_| false)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProjectContract {
    pub address: Addr,
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
    pub fn to_humanized(&self, api: &dyn Api) -> StdResult<ContractInfo> {
        let info = ContractInfo {
            address: api.addr_humanize(&self.address)?,
            code_hash: self.code_hash.clone(),
        };
        Ok(info)
    }
}

pub fn project_count(storage: &dyn Storage) -> StdResult<u32> {
    return PROJECTS_STORE.get_len(storage);
}

pub fn add_project(storage: &mut dyn Storage, project: StoredContractInfo) -> StdResult<u32> {
    PROJECTS_STORE.push(storage, &project)?;
    project_count(storage).map(|len| len-1)
}

pub fn get_projects(
    storage: &dyn Storage,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<StoredContractInfo>> {
    // Take `page_size` projects starting from the latest project, potentially skipping `page * page_size`
    // projects from the start.
    let projects = PROJECTS_STORE
        .iter(storage)?
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _)
        .collect();
    projects
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
