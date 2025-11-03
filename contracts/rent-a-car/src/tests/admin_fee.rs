use soroban_sdk::{testutils::Address as _, Address};
use crate::{
    tests::config::contract::ContractTest,
    storage::{rental::read_rental, admin::read_admin_balance, owner::read_owner_balance},
};

#[test]
pub fn test_fee_applied_and_balances() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    let renter = Address::generate(&env);
    let price_per_day = 1000_i128;
    let total_days = 2_u32;
    let amount = 2000_i128;
    let fee = 300_i128;
    let deposit_total = amount + fee;

    // Fondear renter con tokens reales
    token_admin.mint(&renter, &deposit_total);

    contract.add_car(&owner, &price_per_day);
    contract.set_admin_fee(&fee);
    contract.rental(&renter, &owner, &total_days, &amount);
    let rental = env.as_contract(&contract.address, || read_rental(&env, &renter, &owner));
    assert_eq!(rental.amount, amount);
    assert_eq!(rental.fee_applied, fee);
    assert_eq!(rental.deposit_total, amount + fee);

    let admin_bal = env.as_contract(&contract.address, || read_admin_balance(&env));
    assert_eq!(admin_bal, fee);

    let owner_bal = env.as_contract(&contract.address, || read_owner_balance(&env, &owner));
    assert_eq!(owner_bal, amount);
}
