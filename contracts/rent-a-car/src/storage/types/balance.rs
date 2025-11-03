use soroban_sdk::contracttype;

#[derive(Clone)]
#[contracttype]
pub struct Balance(pub i128);