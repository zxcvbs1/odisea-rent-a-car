# Rent a Car en Soroban (Stellar)

Este proyecto es un contrato inteligente para una dApp de alquiler de autos sobre Soroban (la VM de contratos de Stellar). Implementa un flujo simple con cobro de fee del admin, deposito del renter, y retiros controlados por estado del auto.

## Idea principal
- Admin configura el contrato y el token de pago.
- Admin publica autos (uno por duenio) con un precio por dia.
- Cualquier usuario puede alquilar un auto haciendo un deposito = monto + fee.
- El admin puede retirar sus fees cuando quiera.
- El duenio solo puede retirar cuando el auto fue devuelto.

## Roles
- Admin: direccion que inicializa, configura el fee y puede retirar fees.
- Duenio (owner): publica su auto y retira lo que le corresponde del alquiler.
- Renter: alquila el auto y transfiere los fondos al contrato.

## Token
- El contrato usa un token configurado al desplegar (puede ser XLM nativo u otro SAC).
- El balance real del contrato se consulta directamente al token (`get_contract_balance`). No se guarda un ContractBalance propio.

## Almacenamiento (claves y tipos)
- Admin, Token
- Car(Address) -> { price_per_day, car_status }
- Rental(Address renter, Address owner) -> { total_days_to_rent, amount, fee_applied, deposit_total }
- AdminFee (fee fijo)
- AdminBalance (acumulado de fees)
- OwnerBalance(Address) (acumulado por duenio)
- CarStatus: Available | Rented | Maintenance


## Reglas de negocio
- Deposito del renter = `amount + fee`.
- `fee` va al saldo del admin, `amount` al saldo del duenio.
- El duenio solo puede retirar cuando el auto esta `Available` (es decir, fue devuelto).
- El admin puede retirar sus fees en cualquier momento (si hay saldo > 0).

## Manejo de errores (Result + enum Error)
El contrato no paniquea en flujos publicos: devuelve `Result` con un `Error` tipado. Algunos errores:
- `ContractInitialized`, `ContractNotInitialized`, `AdminTokenConflict`
- `AmountMustBePositive`, `RentalDurationCannotBeZero`, `SelfRentalNotAllowed`
- `CarNotFound`, `CarAlreadyExist`, `CarAlreadyRented`, `CarNotRented`, `CarStillRented`
- `InsufficientBalance`, `OverflowError`, `TokenNotFound`, `AdminNotFound`

Notas:
- Se valida init con `ensure_initialized`.
- Se hace prechequeo de saldo del renter antes de transferir.
- Se usan `checked_add` para evitar overflow en `amount + fee`.

## Como se prueban los errores
El cliente generado por soroban-sdk expone dos variantes por funcion:
- `foo(...)` devuelve el Ok directo y paniquea si el contrato retorna Err.
- `try_foo(...)` devuelve `Result<OkType, Result<ContractError, InvokeError>>`.

Para asertar el error exacto en tests se usa `try_*` y se compara el `ContractError`. Ejemplo simple:
```rust
let inner = contract.try_get_car_status(&owner).unwrap_err();
let actual = inner.expect("se esperaba ContractError");
assert_eq!(actual, ContractError::CarNotFound);
```
Tambien hay tests de flujo que usan `#[should_panic]` cuando alcanza con validar que falla.

## Tests incluidos
- initialize, add_car, get_car_status, rental
- admin_fee: valida fee aplicado y saldos
- withdrawals: flujo de retiros (y caso que debe fallar si el auto no fue devuelto)
- withdraw_flags: helpers `can_*` y balances
- aliases: mapea deposit/withdraw/payout_owner y checa balances
- errors: suite de validaciones con `try_*` y asertos de `ContractError`

Ejemplos para correr:
- Todos: `cargo test`
- Un modulo: `cargo test errors::`
- Un test exacto: `cargo test tests::errors::error_get_car_status_not_found -- --exact`
- Mostrar prints (si habilitas std en tests): `-- --nocapture`

## Scripts utiles
- `scripts/deploy.ps1`: build, optimize, deploy y escribe `.env` y `.env.contract` con datos utilitarios.
- `scripts/e2e-test.ps1`: script end to end contra testnet (requiere cli de stellar y variables).

## Build
- SDK: `soroban-sdk = 22.0.0`
- Tipo de crate: `cdylib` para compilar a wasm
- Build wasm: `cargo build --target wasm32v1-none --release`

## Decisiones de diseno
- No se guarda un ContractBalance interno: se confia en el balance real del token del contrato.
- Balance por duenio se guarda separado (no dentro de `Car`).
- Fee fijo (no porcentual). El soporte de porcentajes podria agregarse luego.
- Un auto por duenio en este modelo. Multi auto requeriria extender claves (por ejemplo, car_id).

## Futuro posible
- Retiros parciales por monto
- Multi auto por duenio
- Fee por porcentaje (bps)
- Mas estados de auto y flujos (maintenance real)

---

Este contrato modela un alquiler simple con deposito + fee, retiros controlados, y errores tipados para una integracion clara con la dApp y los tests.

