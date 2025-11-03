use soroban_sdk::{testutils::Address as _, Address};
use crate::{
    tests::config::contract::ContractTest,
    storage::{
        owner::write_owner_balance,
        admin::write_admin_balance,
    },
    storage::types::error::Error as ContractError,
};

#[test]
pub fn secure_add_balances_overflow() {
    // Caso 1: overflow en balance del owner (owner_balance + amount)
    {
        let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();
        let owner = Address::generate(&env);
        let renter = Address::generate(&env);

        contract.add_car(&owner, &1000);
        contract.set_admin_fee(&0); // sin fee

        // Prepara balance del owner al máximo
        env.as_contract(&contract.address, || {
            write_owner_balance(&env, &owner, i128::MAX);
        });

        // Fondeo renter y rental que intenta sumar 1000 al balance del owner → overflow
        token_admin.mint(&renter, &1000);
        let err = contract.try_rental(&renter, &owner, &1, &1000).unwrap_err();
        let contract_err = err.expect("unexpected invoke error");
        assert_eq!(contract_err, ContractError::OverflowError);
    }

    // Caso 2: overflow en balance del admin (admin_balance + fee)
    {
        let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();
        let owner = Address::generate(&env);
        let renter = Address::generate(&env);

        contract.add_car(&owner, &1000);
        contract.set_admin_fee(&2); // fee 2

        // Lleva el balance admin a MAX-1, sumarle 2 → overflow
        env.as_contract(&contract.address, || {
            write_admin_balance(&env, i128::MAX - 1);
        });

        token_admin.mint(&renter, &1002); // amount 1000 + fee 2
        let err = contract.try_rental(&renter, &owner, &1, &1000).unwrap_err();
        let contract_err = err.expect("unexpected invoke error");
        assert_eq!(contract_err, ContractError::OverflowError);
    }
}