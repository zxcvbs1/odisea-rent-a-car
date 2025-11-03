use soroban_sdk::{Address, Env, Symbol};

pub(crate) fn admin_fee_set(env: &Env, admin: Address, new_fee: i128) {
    let topics = (Symbol::new(env, "admin_fee_set"), admin);
    env.events().publish(topics, new_fee);
}