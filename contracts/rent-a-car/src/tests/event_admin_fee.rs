use soroban_sdk::{
    testutils::Address as _,
    vec, Address, IntoVal, Symbol, TryFromVal,
};
use crate::tests::config::{contract::ContractTest, utils::get_contract_events};

#[test]
pub fn event_admin_fee_set_emits_correct_topics_and_data() {
    let ContractTest { env, contract, .. } = ContractTest::setup();

    // Arrange
    let admin = contract.get_admin();
    let new_fee: i128 = 123;

    // Act
    contract.set_admin_fee(&new_fee);

    // Assert: find admin_fee_set event for this contract
    let events = get_contract_events(&env, &contract.address);

    let found = events.iter().find(|(addr, topics, _data)| {
        *addr == contract.address
            && *topics
                == vec![
                    &env,
                    *Symbol::new(&env, "admin_fee_set").as_val(),
                    admin.clone().into_val(&env),
                ]
    });

    assert!(found.is_some(), "admin_fee_set event not found");

    // Check emitted payload is the fee value
    let (_addr, _topics, data) = found.unwrap();
    let emitted_fee: i128 = i128::try_from_val(&env, &data).expect("invalid fee payload");
    assert_eq!(emitted_fee, new_fee);
}