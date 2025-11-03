use soroban_sdk::{Address, Env};
use crate::storage::types::{car_status::CarStatus, error::Error};

pub trait RentACarContractTrait {
    // Constructor y admin
    fn __constructor(env: &Env, admin: Address, token: Address) -> Result<(), Error>;
    fn get_admin(env: &Env) -> Address;

    // Autos
    fn add_car(env: &Env, owner: Address, price_per_day: i128) -> Result<(), Error>;
    fn get_car_status(env: &Env, owner: Address) -> Result<CarStatus, Error>;
    fn remove_car(env: &Env, owner: Address) -> Result<(), Error>;

    // Comisiones
    fn set_admin_fee(env: &Env, fee: i128) -> Result<(), Error>;
    fn get_admin_fee(env: &Env) -> i128;

    // Flujo alquiler / retiros
    fn rental(env: &Env, renter: Address, owner: Address, total_days_to_rent: u32, amount: i128) -> Result<(), Error>;
    fn return_car(env: &Env, renter: Address, owner: Address) -> Result<(), Error>;
    fn withdraw_admin(env: &Env) -> Result<i128, Error>;
    fn withdraw_owner(env: &Env, owner: Address) -> Result<i128, Error>;

    // Helpers dApp
    fn get_owner_balance(env: &Env, owner: Address) -> i128;
    fn get_admin_balance(env: &Env) -> i128;
    fn can_owner_withdraw(env: &Env, owner: Address) -> bool;
    fn can_admin_withdraw(env: &Env) -> bool;

    // Aliases (compat)
    fn deposit(env: &Env, renter: Address, owner: Address, total_days_to_rent: u32, amount: i128) -> Result<(), Error>;
    fn withdraw(env: &Env, owner: Address) -> Result<i128, Error>;
    fn payout_owner(env: &Env, owner: Address) -> Result<i128, Error>;

    // Consulta de balance del contrato en el token
    fn get_contract_balance(env: &Env) -> i128;
}