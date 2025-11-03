use soroban_sdk::contracttype;

#[derive(Clone)]
#[contracttype]
pub struct Rental {
    pub total_days_to_rent: u32,
    // monto base del alquiler (sin fee)
    pub amount: i128,
    // nuevo: fee aplicado (fijo)
    pub fee_applied: i128,
    // nuevo: deposito total = amount + fee_applied
    pub deposit_total: i128,
}