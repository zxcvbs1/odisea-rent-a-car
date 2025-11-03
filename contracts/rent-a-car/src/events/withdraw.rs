use soroban_sdk::{Address, Env, Symbol};
use crate::storage::types::car_status::CarStatus;

// Owner: habilitado para retirar (auto Available + balance > 0)
pub(crate) fn owner_withdraw_ready(env: &Env, owner: Address, balance: i128, car_status: CarStatus) {
    let topics = (Symbol::new(env, "owner_withdraw_ready"), owner);
    env.events().publish(topics, (balance, car_status));
}

// Owner: retiro ejecutado
pub(crate) fn owner_withdraw_executed(env: &Env, owner: Address, amount: i128) {
    let topics = (Symbol::new(env, "owner_withdraw_executed"), owner);
    env.events().publish(topics, amount);
}

// Admin: habilitado para retirar fee (fee_balance > 0)
pub(crate) fn admin_withdraw_ready(env: &Env, admin: Address, fee_balance: i128) {
    let topics = (Symbol::new(env, "admin_withdraw_ready"), admin);
    env.events().publish(topics, fee_balance);
}

// Admin: retiro ejecutado
pub(crate) fn admin_withdraw_executed(env: &Env, admin: Address, amount: i128) {
    let topics = (Symbol::new(env, "admin_withdraw_executed"), admin);
    env.events().publish(topics, amount);
}