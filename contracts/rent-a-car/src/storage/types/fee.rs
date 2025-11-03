use soroban_sdk::contracttype;

#[derive(Clone)]
#[contracttype]
pub enum FeeMode {
    Flat(i128),
    // A implementar porcentajes: Bps(u32),
}

#[derive(Clone)]
#[contracttype]
pub struct FeeConfig {
    pub mode: FeeMode,
}