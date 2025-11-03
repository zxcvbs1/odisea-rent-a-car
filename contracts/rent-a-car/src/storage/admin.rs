use super::types::{
    balance::Balance,
    fee::{FeeConfig, FeeMode},
    storage::DataKey,
};
use soroban_sdk::{Address, Env};
use crate::storage::types::error::Error;


pub(crate) fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}
pub(crate) fn read_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}
pub(crate) fn write_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

// Fee fijo
pub(crate) fn has_admin_fee(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::AdminFee)
}
pub(crate) fn write_fee_config(env: &Env, cfg: &FeeConfig) {
    env.storage().instance().set(&DataKey::AdminFee, cfg);
}
pub(crate) fn read_fee_config(env: &Env) -> FeeConfig {
    if has_admin_fee(env) {
        env.storage().instance().get(&DataKey::AdminFee).unwrap()
    } else {
        FeeConfig {
            mode: FeeMode::Flat(0),
        }
    }
}
pub(crate) fn write_admin_fee(env: &Env, fee: i128) {
    write_fee_config(
        env,
        &FeeConfig {
            mode: FeeMode::Flat(fee),
        },
    );
}
pub(crate) fn read_admin_fee(env: &Env) -> i128 {
    let cfg = read_fee_config(env);
    match cfg.mode {
        FeeMode::Flat(f) => f,
    }
}

// Saldos admin
pub(crate) fn has_admin_balance(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::AdminBalance)
}
pub(crate) fn write_admin_balance(env: &Env, value: i128) {
    env.storage()
        .instance()
        .set(&DataKey::AdminBalance, &Balance(value));
}
pub(crate) fn read_admin_balance(env: &Env) -> i128 {
    if has_admin_balance(env) {
        let b: Balance = env
            .storage()
            .instance()
            .get(&DataKey::AdminBalance)
            .unwrap();
        b.0
    } else {
        0_i128
    }
}
pub(crate) fn add_admin_balance(env: &Env, amount: i128) -> Result<(), Error> {
    let current = read_admin_balance(env);
    let new = current.checked_add(amount).ok_or(Error::OverflowError)?;
    write_admin_balance(env, new);
    Ok(())
}
pub(crate) fn take_admin_balance(env: &Env) -> i128 {
    let amount = read_admin_balance(env);
    write_admin_balance(env, 0_i128);
    amount
}
