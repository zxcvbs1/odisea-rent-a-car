use soroban_sdk::{contract, contractimpl, Address, Env};
use soroban_sdk::token;
use crate::events;

use crate::{
    interfaces::contract::RentACarContractTrait,
    storage::{
        admin::{
            has_admin, read_admin, write_admin,
            write_admin_fee, read_admin_fee,
            add_admin_balance, write_admin_balance, take_admin_balance,
            read_admin_balance,
        },
        car::{has_car, read_car, remove_car as remove_car_storage, write_car},
        rental::{has_rental, write_rental, remove_rental},
        structs::{car::Car, rental::Rental},
        token::{write_token, read_token},
        types::{car_status::CarStatus, error::Error, storage::DataKey},
        owner::{
            add_owner_balance,
            take_owner_balance,
            read_owner_balance,
            has_owner_balance,
        },
    }
};

#[contract]
pub struct RentACarContract;

fn ensure_initialized(env: &Env) -> Result<(), Error> {
    if !has_admin(env) {
        return Err(Error::ContractNotInitialized);
    }
    if !env.storage().instance().has(&DataKey::Token) {
        return Err(Error::TokenNotFound);
    }
    Ok(())
}

#[contractimpl]
impl RentACarContractTrait for RentACarContract {
    fn __constructor(env: &Env, admin: Address, token: Address) -> Result<(), Error> {
        if admin == token {
            return Err(Error::AdminTokenConflict);
        }
        if has_admin(env) {
            return Err(Error::ContractInitialized);
        }
        write_admin(env, &admin);
        write_token(env, &token);

        // inicializa fee y balance admin en 0
        write_admin_fee(env, 0_i128);
        write_admin_balance(env, 0_i128);
        events::contract::contract_initialized(env, admin, token);
        Ok(())
    }

    fn get_admin(env: &Env) -> Address {
        read_admin(env)
    }

    fn add_car(env: &Env, owner: Address, price_per_day: i128) -> Result<(), Error> {
        ensure_initialized(env)?;
        let admin = read_admin(env);
        admin.require_auth();


        if price_per_day <= 0 {
            return Err(Error::AmountMustBePositive);
        }
        if has_car(env, &owner) {
            return Err(Error::CarAlreadyExist);
        }
        let car = Car { price_per_day, car_status: CarStatus::Available };
        write_car(env, &owner, &car);
        events::car::car_added(env, owner, price_per_day);

        Ok(())
    }

    fn get_car_status(env: &Env, owner: Address) -> Result<CarStatus, Error> {
        ensure_initialized(env)?;
        if !has_car(env, &owner) {
            return Err(Error::CarNotFound);
        }
        let car = read_car(env, &owner);
        Ok(car.car_status)
    }

    fn set_admin_fee(env: &Env, fee: i128) -> Result<(), Error> {
        ensure_initialized(env)?;

        let admin = read_admin(env);
        admin.require_auth();
        if fee < 0 {
            return Err(Error::AmountMustBePositive);
        }
        write_admin_fee(env, fee);
        
        events::admin::admin_fee_set(env, admin, fee);
        Ok(())
    }

    fn get_admin_fee(env: &Env) -> i128 {
        read_admin_fee(env)
    }

    fn rental(env: &Env, renter: Address, owner: Address, total_days_to_rent: u32, amount: i128) -> Result<(), Error> {
        ensure_initialized(env)?;
        renter.require_auth();

        if amount <= 0 {
            return Err(Error::AmountMustBePositive);
        }
        if total_days_to_rent == 0 {
            return Err(Error::RentalDurationCannotBeZero);
        }
        if renter == owner {
            return Err(Error::SelfRentalNotAllowed);
        }
        if !has_car(env, &owner) {
            return Err(Error::CarNotFound);
        }

        let mut car = read_car(env, &owner);
        if car.car_status != CarStatus::Available {
            return Err(Error::CarAlreadyRented);
        }

        let fee = read_admin_fee(env);
        let deposit_total = amount.checked_add(fee).ok_or(Error::OverflowError)?;

        // pre-check de saldo
        let t = token::Client::new(env, &read_token(env));
        let renter_balance = t.balance(&renter);
        if renter_balance < deposit_total {
            return Err(Error::InsufficientBalance);
        }

        // Persistencia
        car.car_status = CarStatus::Rented;

        let rental = Rental {
            total_days_to_rent,
            amount,        // base sin fee
            fee_applied: fee,
            deposit_total, // base + fee
        };
        let prev_admin_bal = read_admin_balance(env);
        if fee > 0 {
            add_admin_balance(env, fee)?; // usando checked_add internamente
            if prev_admin_bal == 0 {
                let admin = read_admin(env);
                let new_bal = prev_admin_bal.checked_add(fee).ok_or(Error::OverflowError)?;
                events::withdraw::admin_withdraw_ready(env, admin, new_bal);
            }
        }

        add_owner_balance(env, &owner, amount)?;

        write_car(env, &owner, &car);
        write_rental(env, &renter, &owner, &rental);

        // Transferencia de tokens: renter -> contrato
        t.transfer(&renter, &env.current_contract_address(), &deposit_total);

        events::rental::rented(env, renter.clone(), owner.clone(), total_days_to_rent, amount, fee, deposit_total);
        Ok(())
    }

fn return_car(env: &Env, renter: Address, owner: Address) -> Result<(), Error> {
    ensure_initialized(env)?;
    renter.require_auth();

    if !has_car(env, &owner) {
        return Err(Error::CarNotFound);
    }
    let mut car = read_car(env, &owner);
    if car.car_status != CarStatus::Rented {
        return Err(Error::CarNotRented);
    }

    if !has_rental(env, &renter, &owner) {
        return Err(Error::CarNotRented);
    }

    car.car_status = CarStatus::Available;
    write_car(env, &owner, &car);
    remove_rental(env, &renter, &owner);
    let bal = if has_owner_balance(env, &owner) { read_owner_balance(env, &owner) } else { 0 };
    if bal > 0 { events::withdraw::owner_withdraw_ready(env, owner.clone(), bal, CarStatus::Available); }




    events::rental::car_returned(env, renter, owner);
    Ok(())
}

    fn withdraw_admin(env: &Env) -> Result<i128, Error> {
        ensure_initialized(env)?;
        let admin = read_admin(env);
        admin.require_auth();

        let amount = take_admin_balance(env);
        if amount > 0 {
            let t = token::Client::new(env, &read_token(env));
            t.transfer(&env.current_contract_address(), &admin, &amount);
            events::withdraw::admin_withdraw_executed(env, admin, amount);



        }

        
        Ok(amount)
    }

    fn withdraw_owner(env: &Env, owner: Address) -> Result<i128, Error> {
        ensure_initialized(env)?;
        owner.require_auth();

        if !has_car(env, &owner) {
            return Err(Error::CarNotFound);
        }
        let car = read_car(env, &owner);
        if car.car_status != CarStatus::Available {
            return Err(Error::CarStillRented);
        }

        let amount = take_owner_balance(env, &owner);
        if amount > 0 {
            let t = token::Client::new(env, &read_token(env));
            t.transfer(&env.current_contract_address(), &owner, &amount);
            events::withdraw::owner_withdraw_executed(env, owner, amount);

        }

        
        Ok(amount)
    }

    fn remove_car(env: &Env, owner: Address) -> Result<(), Error> {
        ensure_initialized(env)?;
        let admin = read_admin(env);
        admin.require_auth();
        if !has_car(env, &owner) {
            return Err(Error::CarNotFound);
        }
        let car = read_car(env, &owner);
        if car.car_status != CarStatus::Available {
            return Err(Error::CarStillRented);
        }
        let pending = if has_owner_balance(env, &owner) {
        read_owner_balance(env, &owner)
    } else {
        0_i128
    };
    if pending > 0 {
        return Err(Error::OwnerBalancePending);
    }

        remove_car_storage(env, &owner);

        events::car::car_removed(env, owner);
        Ok(())
    }

    // Helpers (sin cambios de firmas)
    fn get_owner_balance(env: &Env, owner: Address) -> i128 {
        if has_owner_balance(env, &owner) { read_owner_balance(env, &owner) } else { 0 }
    }

    fn get_admin_balance(env: &Env) -> i128 {
        read_admin_balance(env)
    }

    fn can_owner_withdraw(env: &Env, owner: Address) -> bool {
        if !has_car(env, &owner) { return false; }
        let car = read_car(env, &owner);
        if car.car_status != CarStatus::Available { return false; }
        let bal = if has_owner_balance(env, &owner) { read_owner_balance(env, &owner) } else { 0 };
        bal > 0
    }

    fn can_admin_withdraw(env: &Env) -> bool {
        read_admin_balance(env) > 0
    }

    // Aliases
    fn deposit(env: &Env, renter: Address, owner: Address, total_days_to_rent: u32, amount: i128) -> Result<(), Error> {
        Self::rental(env, renter, owner, total_days_to_rent, amount)
    }

    fn withdraw(env: &Env, owner: Address) -> Result<i128, Error> {
        Self::withdraw_owner(env, owner)
    }

    fn payout_owner(env: &Env, owner: Address) -> Result<i128, Error> {
        Self::withdraw_owner(env, owner)
    }

    fn get_contract_balance(env: &Env) -> i128 {
        let t = token::Client::new(env, &read_token(env));
        t.balance(&env.current_contract_address())
    }
}