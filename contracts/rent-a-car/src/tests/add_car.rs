use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    IntoVal, Address,
};
use crate::tests::config::contract::ContractTest;
use crate::{storage::{car::read_car, types::car_status::CarStatus},};
#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_unauthorized_user_cannot_add_car() {
    // Arrange
    let ContractTest { env, contract, .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    let price: i128 = 1000;

    // Usuario que NO es el admin del contrato
    let fake_admin = Address::generate(&env);

    // Act: forzamos que la única firma sea de fake_admin,
    // pero el contrato exige que firme el admin real.
    contract
        .mock_auths(&[MockAuth {
            address: &fake_admin,
            invoke: &MockAuthInvoke {
                contract: &contract.address,
                fn_name: "add_car",
                args: (owner.clone(), price).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .add_car(&owner, &price); // Debe panicar con Error(Auth, InvalidAction)
}

#[test]
fn test_admin_can_add_car_authenticated() {
    // Arrange
    let ContractTest { env, contract, admin, .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    let price: i128 = 1500;

    // Act: firmando como admin real no debe fallar
    contract
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract.address,
                fn_name: "add_car",
                args: (owner.clone(), price).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .add_car(&owner, &price);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn renter_no_puede_retirar_fondos_de_owner() {
    let ContractTest { env, contract, .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    let renter = Address::generate(&env);

    // Opcional: garantizar que el owner existe como auto (no afecta al fallo por auth).
    contract.add_car(&owner, &1000);

    // Firma el renter pero el contrato exige firma del owner → Auth error.
    contract
        .mock_auths(&[MockAuth {
            address: &renter,
            invoke: &MockAuthInvoke {
                contract: &contract.address,
                fn_name: "withdraw_owner",
                args: (owner.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .withdraw_owner(&owner);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn owner_no_puede_retirar_fondos_de_otro_owner() {
    let ContractTest { env, contract, .. } = ContractTest::setup();

    let owner_a = Address::generate(&env);
    let owner_b = Address::generate(&env);

    // Opcional: registrar el auto de owner_a.
    contract.add_car(&owner_a, &1000);
    contract.add_car(&owner_b, &500);


    // Firma owner_b intentando retirar del balance de owner_a → Auth error.
    contract
        .mock_auths(&[MockAuth {
            address: &owner_b,
            invoke: &MockAuthInvoke {
                contract: &contract.address,
                fn_name: "withdraw_owner",
                args: (owner_a.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .withdraw_owner(&owner_a);
}

#[test]
pub fn test_add_car_successfully() {
    let ContractTest { env, contract, .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    let price_per_day = 1500_i128;

    contract.add_car( &owner, &price_per_day);
    let stored_car = env.as_contract(&contract.address, || {
        read_car(&env, &owner)
    });

    assert_eq!(stored_car.price_per_day, price_per_day);
    assert_eq!(stored_car.car_status, CarStatus::Available);
}
