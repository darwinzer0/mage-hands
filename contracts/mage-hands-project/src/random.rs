use cosmwasm_std::{StdResult, Storage,};
use cosmwasm_storage::{ReadonlySingleton, Singleton};

use rand::{SeedableRng};
use rand_chacha::ChaChaRng;
use sha2::{Digest, Sha256};

static KEY_ENTROPY_POOL: &[u8] = b"entropy_pool";

fn get_current_entropy_pool(storage: &dyn Storage) -> [u8; 32] {
    ReadonlySingleton::new(storage, KEY_ENTROPY_POOL)
        .load()
        .or::<[u8; 32]>(Ok([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]))
        .unwrap()
}

pub fn supply_more_entropy(
    storage: &mut dyn Storage,
    additional_entropy: &[u8],
) -> StdResult<()> {
    let current_entropy_pool = get_current_entropy_pool(storage);
    println!("{:?}", current_entropy_pool);

    let mut new_entropy_source = Vec::from(current_entropy_pool);
    new_entropy_source.extend(additional_entropy);

    let new_entropy_pool: [u8; 32] = Sha256::digest(&new_entropy_source).into();

    Singleton::new(storage, KEY_ENTROPY_POOL).save(&new_entropy_pool)
}

pub fn get_random_number_generator(storage: &dyn Storage) -> ChaChaRng {
    let entropy_pool = get_current_entropy_pool(storage);
    ChaChaRng::from_seed(entropy_pool)
}

pub fn sha_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();

    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_slice());
    result
}
