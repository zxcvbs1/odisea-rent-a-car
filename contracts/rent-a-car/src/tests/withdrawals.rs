
use soroban_sdk::{testutils::Address as _, Address};
use crate::{tests::config::contract::ContractTest, storage::types::error::Error as ContractError};
use crate::storage::{admin::read_admin_balance, owner::read_owner_balance};
#[test]
pub fn test_withdraw_owner_requires_return_specific_error() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    let renter = Address::generate(&env);

    let price_per_day = 1000_i128;
    let total_days = 2_u32;
    let amount = 2000_i128;
    let fee = 100_i128;

    token_admin.mint(&renter, &(amount + fee));
    contract.add_car(&owner, &price_per_day);
    contract.set_admin_fee(&fee);
    contract.rental(&renter, &owner, &total_days, &amount);

    let err = contract.try_withdraw_owner(&owner).unwrap_err();      // Err(Result<ContractError, InvokeError>)
    let contract_err = err.expect("unexpected invoke error");        // -> ContractError
    assert_eq!(contract_err, ContractError::CarStillRented);
    
}


#[test]
pub fn test_withdraw_owner_after_return_and_admin_withdraw() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    let renter = Address::generate(&env);
    let price_per_day = 1500_i128;
    let total_days = 3_u32;
    let amount = 4500_i128;
    let fee = 200_i128;
    let deposit_total = amount + fee;

    token_admin.mint(&renter, &deposit_total);

    contract.add_car(&owner, &price_per_day);
    contract.set_admin_fee(&fee);
    contract.rental(&renter, &owner, &total_days, &amount);

    let admin_taken = contract.withdraw_admin();
    assert_eq!(admin_taken, fee);
    let admin_bal = env.as_contract(&contract.address, || read_admin_balance(&env));
    assert_eq!(admin_bal, 0);

    contract.return_car(&renter, &owner);

    let owner_taken = contract.withdraw_owner(&owner);
    assert_eq!(owner_taken, amount);
    let owner_bal = env.as_contract(&contract.address, || read_owner_balance(&env, &owner));
    assert_eq!(owner_bal, 0);
}
