use soroban_sdk::{testutils::Address as _, Address};

use crate::tests::config::contract::ContractTest;

#[test]
pub fn test_aliases_flow_and_contract_balance() {
    let ContractTest { env, contract, token: (_, token_admin, _), .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    let renter = Address::generate(&env);

    let price_per_day = 1500_i128;
    let total_days = 3_u32;
    let amount = 4500_i128;
    let fee = 100_i128;
    let deposit_total = amount + fee;

    // Fondear al renter con el depósito total
    token_admin.mint(&renter, &deposit_total);

    // Setup del auto y fee
    contract.add_car(&owner, &price_per_day);
    contract.set_admin_fee(&fee);

    // Alias deposit → debe mapear a rental
    contract.deposit(&renter, &owner, &total_days, &amount);

    // Balance del contrato (token) debe ser el depósito total
    let bal1 = contract.get_contract_balance();
    assert_eq!(bal1, deposit_total);

    // Retiro admin (alias no existe, usamos withdraw_admin real)
    let admin_taken = contract.withdraw_admin();
    assert_eq!(admin_taken, fee);

    // Balance del contrato ahora = amount
    let bal2 = contract.get_contract_balance();
    assert_eq!(bal2, amount);

    // Devolver auto
    contract.return_car(&renter, &owner);

    // Alias withdraw → debe mapear a withdraw_owner
    let owner_taken = contract.withdraw(&owner);
    assert_eq!(owner_taken, amount);

    // Contrato en 0
    let bal3 = contract.get_contract_balance();
    assert_eq!(bal3, 0);

    // Alias payout_owner → idempotente aquí (no hay saldo), debe devolver 0
    let again = contract.payout_owner(&owner);
    assert_eq!(again, 0);
}
