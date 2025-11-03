use soroban_sdk::{testutils::Address as _, Address};
use crate::tests::config::contract::ContractTest;

#[test]
pub fn test_flags_and_balances() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    let renter = Address::generate(&env);
    let price_per_day = 1500_i128;
    let total_days = 3_u32;
    let amount = 4500_i128;
    let fee = 100_i128;
    let deposit_total = amount + fee;

    // Fondeo renter y setup
    token_admin.mint(&renter, &deposit_total);
    contract.add_car(&owner, &price_per_day);
    contract.set_admin_fee(&fee);

    // Antes de alquilar: owner no puede retirar
    assert_eq!(contract.get_owner_balance(&owner), 0);
    assert!(!contract.can_owner_withdraw(&owner));
    assert_eq!(contract.get_admin_balance(), 0);
    assert!(!contract.can_admin_withdraw());

    // Alquilar
    contract.rental(&renter, &owner, &total_days, &amount);

    // Luego del rental: admin tiene fee y puede retirar, owner aún no puede retirar
    assert_eq!(contract.get_admin_balance(), fee);
    assert!(contract.can_admin_withdraw());
    assert_eq!(contract.get_owner_balance(&owner), amount);
    assert!(!contract.can_owner_withdraw(&owner)); // auto está Rented

    // Retiro admin
    let admin_taken = contract.withdraw_admin();
    assert_eq!(admin_taken, fee);
    assert_eq!(contract.get_admin_balance(), 0);
    assert!(!contract.can_admin_withdraw());

    // Devolver auto → ahora owner puede retirar
    contract.return_car(&renter, &owner);
    assert!(contract.can_owner_withdraw(&owner));

    // Retiro owner
    let owner_taken = contract.withdraw_owner(&owner);
    assert_eq!(owner_taken, amount);
    assert_eq!(contract.get_owner_balance(&owner), 0);
    assert!(!contract.can_owner_withdraw(&owner));
}
