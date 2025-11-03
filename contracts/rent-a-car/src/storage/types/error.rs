use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    ContractInitialized = 0,
    ContractNotInitialized = 1,
    CarNotFound = 2,
    AdminTokenConflict = 3,
    RentalDurationCannotBeZero = 4,
    CarAlreadyExist = 5,
    AmountMustBePositive = 6,
    InsufficientBalance = 7,
    SelfRentalNotAllowed = 8,
    CarAlreadyRented = 9,
    TokenNotFound = 10,
    AdminNotFound = 11,
    OverflowError = 12,
    UnderFlowError = 13,
    CarStillRented = 14,
    CarNotRented = 15,
}