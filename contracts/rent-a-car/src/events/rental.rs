use soroban_sdk::{Address, Env, Symbol};

pub(crate) fn rented(
    env: &Env,
    renter: Address,
    owner: Address,
    total_days_to_rent: u32,
    amount: i128,
    fee_applied: i128,
    deposit_total: i128,
) {
    let topics = (Symbol::new(env, "rented"), renter.clone(), owner.clone());
    env.events().publish(
        topics,
        (total_days_to_rent, amount, fee_applied, deposit_total),
    );
}

pub(crate) fn car_returned(env: &Env, renter: Address, owner: Address) {
    let topics = (Symbol::new(env, "car_returned"), renter.clone(), owner.clone());
    env.events().publish(topics, ());
}