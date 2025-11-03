use soroban_sdk::{Address, Env};
use crate::storage::types::{storage::DataKey, balance::Balance};

pub(crate) fn has_owner_balance(env: &Env, owner: &Address) -> bool {
    env.storage().instance().has(&DataKey::OwnerBalance(owner.clone()))
}

pub(crate) fn read_owner_balance(env: &Env, owner: &Address) -> i128 {
    let b: Balance = env.storage().instance().get(&DataKey::OwnerBalance(owner.clone())).unwrap();
    b.0
}

pub(crate) fn write_owner_balance(env: &Env, owner: &Address, value: i128) {
    env.storage()
        .instance()
        .set(&DataKey::OwnerBalance(owner.clone()), &Balance(value));
}



pub(crate) fn add_owner_balance(env: &Env, owner: &Address, amount: i128) {
    if has_owner_balance(env, owner) {
        let current = read_owner_balance(env, owner);
        write_owner_balance(env, owner, current + amount);
    } else {
        write_owner_balance(env, owner, amount);
    }
}



pub(crate) fn take_owner_balance(env: &Env, owner: &Address) -> i128 {
    let amount = if has_owner_balance(env, owner) { read_owner_balance(env, owner) } else { 0_i128 };
    write_owner_balance(env, owner, 0_i128);
    amount
}