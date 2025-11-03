use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    token,
    Address,
    Env,
    IntoVal,
};

use crate::{contract::RentACarContractClient, RentACarContract};
use crate::tests::config::utils::create_token_contract;

pub struct ContractTest<'a> {
    pub env: Env,
    pub contract: RentACarContractClient<'a>,
    pub admin: Address,
    pub token: (token::Client<'a>, token::StellarAssetClient<'a>, Address),
}

impl<'a> ContractTest<'a> {
    pub fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths(); // habilita require_auth() en tests
        let admin = Address::generate(&env);
        let token_issuer = Address::generate(&env);

        let (token_client, token_admin) = create_token_contract(&env, &token_issuer);

        let contract_id = env.register(RentACarContract, (&admin, &token_client.address));
        let contract = RentACarContractClient::new(&env, &contract_id);

        ContractTest {
            env,
            contract,
            admin,
            token: (token_client, token_admin, token_issuer),
        }
    }
}


#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
pub fn test_owner_no_puede_remove_car() {
    let ContractTest { env, contract, .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    contract.add_car(&owner, &1000);

    // Firma el owner, pero remove_car exige firma del admin → Auth error
    contract
        .mock_auths(&[MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &contract.address,
                fn_name: "remove_car",
                args: (owner.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .remove_car(&owner);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
pub fn test_renter_no_puede_remove_car() {
    let ContractTest { env, contract, .. } = ContractTest::setup();

    let owner = Address::generate(&env);
    let renter = Address::generate(&env);
    contract.add_car(&owner, &1000);

    // Firma el renter, pero remove_car exige firma del admin → Auth error
    contract
        .mock_auths(&[MockAuth {
            address: &renter,
            invoke: &MockAuthInvoke {
                contract: &contract.address,
                fn_name: "remove_car",
                args: (owner.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .remove_car(&owner);
}