use soroban_sdk::{ testutils::Address as _, vec, Address, IntoVal, Symbol, TryFromVal };

use crate::tests::config::{contract::ContractTest, utils::get_contract_events};
use crate::storage::token::read_token;

use crate::storage::types::car_status::CarStatus;



#[test]
pub fn event_admin_fee_set() {
    let ContractTest { env, contract, admin, .. } = ContractTest::setup();
    let token_addr = env.as_contract(&contract.address, || read_token(&env));

    let fee = 123_i128;
    contract.set_admin_fee(&fee);

    let events = get_contract_events(&env, &contract.address);

    // Check admin_fee_set exists (don’t assert full history)
    let found = events.iter().any(|(addr, topics, data)| {
        if addr != contract.address {
            return false;
        }
        if topics
            != vec![
                &env,
                *Symbol::new(&env, "admin_fee_set").as_val(),
                admin.clone().into_val(&env),
            ]
        {
            return false;
        }
        let amt: i128 = i128::try_from_val(&env, &data).unwrap();
        amt == fee
    });
    assert!(found, "admin_fee_set not found");

    // No validamos contract_initialized aquí por la nota anterior.
}

#[test]
pub fn event_car_added() {
    let ContractTest { env, contract, admin, .. } = ContractTest::setup();
    let token_addr = env.as_contract(&contract.address, || read_token(&env));

    let owner = Address::generate(&env);
    let price = 1500_i128;
    contract.add_car(&owner, &price);

    let events = get_contract_events(&env, &contract.address);

    let found_added = events.iter().any(|(addr, topics, data)| {
        if addr != contract.address {
            return false;
        }
        if topics
            != vec![
                &env,
                *Symbol::new(&env, "car_added").as_val(),
                owner.clone().into_val(&env),
            ]
        {
            return false;
        }
        let p: i128 = i128::try_from_val(&env, &data).unwrap();
        p == price
    });
    assert!(found_added, "car_added not found");

    // No validamos contract_initialized aquí por la nota anterior.
}

#[test]
pub fn events_rental_includes_admin_ready_and_rented() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    let renter = Address::generate(&env);
    let price = 1500_i128;
    let fee = 100_i128;
    let days = 3_u32;
    let amount = 4500_i128;
    let deposit_total = amount + fee;

    contract.add_car(&owner, &price);
    contract.set_admin_fee(&fee);
    token_admin.mint(&renter, &deposit_total);
    contract.rental(&renter, &owner, &days, &amount);

    let events = get_contract_events(&env, &contract.address);

    // admin_withdraw_ready
    let admin = contract.get_admin();
    let found_ready = events.iter().any(|(addr, topics, data)| {
        if addr != contract.address {
            return false;
        }
        if topics
            != vec![
                &env,
                *Symbol::new(&env, "admin_withdraw_ready").as_val(),
                admin.clone().into_val(&env),
            ]
        {
            return false;
        }
        let amt: i128 = i128::try_from_val(&env, &data).unwrap();
        amt == fee
    });
    assert!(found_ready, "admin_withdraw_ready not found");

    // rented
    let found_rented = events.iter().any(|(addr, topics, data)| {
        if addr != contract.address {
            return false;
        }
        if topics
            != vec![
                &env,
                *Symbol::new(&env, "rented").as_val(),
                renter.clone().into_val(&env),
                owner.clone().into_val(&env),
            ]
        {
            return false;
        }
        let payload: (u32, i128, i128, i128) = <(u32, i128, i128, i128)>::try_from_val(&env, &data).unwrap();
        payload == (days, amount, fee, deposit_total)
    });
    assert!(found_rented, "rented not found");
}

#[test]
pub fn events_return_includes_owner_ready_and_car_returned() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    let renter = Address::generate(&env);
    let price = 1500_i128;
    let fee = 100_i128;
    let days = 2_u32;
    let amount = 3000_i128;
    let deposit_total = amount + fee;

    contract.add_car(&owner, &price);
    contract.set_admin_fee(&fee);
    token_admin.mint(&renter, &deposit_total);
    contract.rental(&renter, &owner, &days, &amount);
    contract.return_car(&renter, &owner);

    let events = get_contract_events(&env, &contract.address);

    // owner_withdraw_ready debe existir con balance y estado Available
    let found_ready = events.iter().any(|(addr, topics, data)| {
        if addr != contract.address {
            return false;
        }
        if topics
            != vec![
                &env,
                *Symbol::new(&env, "owner_withdraw_ready").as_val(),
                owner.clone().into_val(&env),
            ]
        {
            return false;
        }
        let payload: (i128, CarStatus) = <(i128, CarStatus)>::try_from_val(&env, &data).unwrap();
        payload == (amount, CarStatus::Available)
    });
    assert!(found_ready, "owner_withdraw_ready not found");

    // car_returned debe existir para renter/owner
    let found_returned = events.iter().any(|(addr, topics, data)| {
        if addr != contract.address {
            return false;
        }
        if topics
            != vec![
                &env,
                *Symbol::new(&env, "car_returned").as_val(),
                renter.clone().into_val(&env),
                owner.clone().into_val(&env),
            ]
        {
            return false;
        }
        let _unit: () = <()>::try_from_val(&env, &data).unwrap();
        true
    });
    assert!(found_returned, "car_returned not found");
}

