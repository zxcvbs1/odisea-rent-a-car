use soroban_sdk::{testutils::Address as _, Address};
use crate::{
    tests::config::contract::ContractTest,
    storage::types::error::Error as ContractError,
};

// Macro: assert a specific contract error from a try_* client call
macro_rules! assert_contract_err {
    ($res:expr, $expected:path) => {{
        // try_* retorna: Result<OkType, Result<ContractError, soroban_sdk::InvokeError>>
        let inner: Result<ContractError, soroban_sdk::InvokeError> = $res.unwrap_err();
        match inner {
            Ok(actual) => {
                if actual != $expected {
                    panic!(
                        "contract error mismatch: expected {:?}, got {:?}",
                        $expected, actual
                    );
                }
            }
            Err(inv) => {
                panic!(
                    "expected contract error {:?}, got InvokeError: {:?}",
                    $expected, inv
                );
            }
        }
    }};
}

#[test]
pub fn error_set_admin_fee_negative() {
    let ContractTest { contract, .. } = ContractTest::setup();
    assert_contract_err!(contract.try_set_admin_fee(&-1), ContractError::AmountMustBePositive);
}

#[test]
pub fn error_add_car_price_non_positive() {
    let ContractTest { env, contract, .. } = ContractTest::setup();
    let owner = Address::generate(&env);
    assert_contract_err!(contract.try_add_car(&owner, &0), ContractError::AmountMustBePositive);
}

#[test]
pub fn error_rental_amount_non_positive() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();
    let owner = Address::generate(&env);
    let renter = Address::generate(&env);

    contract.add_car(&owner, &1000);
    token_admin.mint(&renter, &1000);

    assert_contract_err!(
        contract.try_rental(&renter, &owner, &1, &0),
        ContractError::AmountMustBePositive
    );
}

#[test]
pub fn error_rental_duration_zero() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();
    let owner = Address::generate(&env);
    let renter = Address::generate(&env);

    contract.add_car(&owner, &1000);
    token_admin.mint(&renter, &2000);

    assert_contract_err!(
        contract.try_rental(&renter, &owner, &0, &1000),
        ContractError::RentalDurationCannotBeZero
    );
}

#[test]
pub fn error_rental_self_rental() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();
    let owner = Address::generate(&env);

    contract.add_car(&owner, &1000);
    token_admin.mint(&owner, &2000);

    assert_contract_err!(
        contract.try_rental(&owner, &owner, &1, &1000),
        ContractError::SelfRentalNotAllowed
    );
}

#[test]
pub fn error_rental_car_not_found() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();
    let owner = Address::generate(&env);  // no add_car
    let renter = Address::generate(&env);

    token_admin.mint(&renter, &2000);

    assert_contract_err!(
        contract.try_rental(&renter, &owner, &1, &1000),
        ContractError::CarNotFound
    );
}

#[test]
pub fn error_rental_car_already_rented() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();
    let owner = Address::generate(&env);
    let renter1 = Address::generate(&env);
    let renter2 = Address::generate(&env);

    contract.add_car(&owner, &1000);
    contract.set_admin_fee(&100);

    let deposit = 1100_i128;
    token_admin.mint(&renter1, &deposit);
    contract.rental(&renter1, &owner, &1, &1000); // rent first time

    token_admin.mint(&renter2, &deposit);
    assert_contract_err!(
        contract.try_rental(&renter2, &owner, &1, &1000),
        ContractError::CarAlreadyRented
    );
}

#[test]
pub fn error_rental_insufficient_balance() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();
    let owner = Address::generate(&env);
    let renter = Address::generate(&env);

    contract.add_car(&owner, &1000);
    contract.set_admin_fee(&100);

    // Mint less than amount + fee (1000 + 100 = 1100)
    token_admin.mint(&renter, &1099);

    assert_contract_err!(
        contract.try_rental(&renter, &owner, &1, &1000),
        ContractError::InsufficientBalance
    );
}

#[test]
pub fn error_return_car_not_rented() {
    let ContractTest { env, contract, .. } = ContractTest::setup();
    let owner = Address::generate(&env);
    let renter = Address::generate(&env);

    contract.add_car(&owner, &1000);

    assert_contract_err!(
        contract.try_return_car(&renter, &owner),
        ContractError::CarNotRented
    );
}

#[test]
pub fn error_remove_car_still_rented() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();
    let owner = Address::generate(&env);
    let renter = Address::generate(&env);

    contract.add_car(&owner, &1000);
    contract.set_admin_fee(&100);
    token_admin.mint(&renter, &1100);
    contract.rental(&renter, &owner, &1, &1000);

    assert_contract_err!(
        contract.try_remove_car(&owner),
        ContractError::CarStillRented
    );
}

#[test]
pub fn error_remove_car_not_found() {
    let ContractTest { env, contract, .. } = ContractTest::setup();
    let owner = Address::generate(&env); // no add_car

    assert_contract_err!(
        contract.try_remove_car(&owner),
        ContractError::CarNotFound
    );
}

#[test]
pub fn error_get_car_status_not_found() {
    let ContractTest { env, contract, .. } = ContractTest::setup();
    let owner = Address::generate(&env); // no add_car

    assert_contract_err!(
        contract.try_get_car_status(&owner),
        ContractError::CarNotFound
    );
}