# PowerShell strict + fail fast
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Paths del repo y crate; usar target del crate
$RepoRoot  = (Resolve-Path (Join-Path $PSScriptRoot ".." )).Path
$CrateDir  = Join-Path $RepoRoot "contracts/rent-a-car"

# 0) Parámetros
$Network      = "testnet"
$Wasm         = "target/wasm32v1-none/release/rent_a_car.wasm"           # si tu build deja rent-a-car.wasm, cámbialo
$WasmOpt      = "target/wasm32v1-none/release/rent_a_car.optimized.wasm"
$PricePerDay  = 1500
$TotalDays    = 3
$Amount       = 4500         # base sin fee
$Fee          = 100          # fee fijo en stroops (unidad mínima)
# Aliases de cuentas locales en stellar CLI
$AdminAlias   = "admin"
$OwnerAlias   = "owner"
$RenterAlias  = "renter"

# Reporte detallado (tablas grandes). Dejalo en $false para resumen simple.
$VerboseReport = $false

function Run($cmd) {
  Write-Host ">> $cmd" -ForegroundColor Cyan
  iex $cmd
}

function GetTokenBalance($sourceAlias, $id) {
  $out = (stellar contract invoke --id $TOKEN_ID --source $sourceAlias --network $Network -- balance --id $id | Out-String)
  $lines = $out -split "`n"
  $numLine = $lines | Where-Object { $_.Trim() -match '^"?-?\d+"?$' } | Select-Object -Last 1
  if (-not $numLine) { return [decimal]0 }
  $numStr = ($numLine -replace '"','').Trim()
  return [decimal]$numStr
}

function ToXlm([decimal]$stroops) {
  return [decimal]::Divide($stroops, 10000000)
}

function Ensure-Key($alias) {
  $out = ""
  try { $out = (stellar keys address $alias | Out-String).Trim() } catch {}
  if (-not $out) {
    Run "stellar keys generate $alias"
  } else {
    Write-Host "Key '$alias' ya existe: $out"
  }
}

# 1) Build + Optimize
$ManifestPath = Join-Path $CrateDir "Cargo.toml"
Run "cargo build --manifest-path `"$ManifestPath`" --target wasm32v1-none --release"
$TargetDir = Join-Path $CrateDir "target/wasm32v1-none/release"
$Wasm      = Join-Path $TargetDir "rent_a_car.wasm"
$WasmOpt   = Join-Path $TargetDir "rent_a_car.optimized.wasm"
Run "stellar contract optimize --wasm $Wasm"

# 2) Claves y fondos
Ensure-Key $AdminAlias
Ensure-Key $OwnerAlias
Ensure-Key $RenterAlias

$ADMIN_ADDR  = (stellar keys address $AdminAlias | Out-String).Trim()
$OWNER_ADDR  = (stellar keys address $OwnerAlias | Out-String).Trim()
$RENTER_ADDR = (stellar keys address $RenterAlias | Out-String).Trim()

# Friendbot funding (idempotente)
Run "stellar keys fund --network $Network $AdminAlias"
Run "stellar keys fund --network $Network $OwnerAlias"
Run "stellar keys fund --network $Network $RenterAlias"

# 3) Token: XLM nativo (SAC ID)
$TOKEN_ID = (stellar contract asset id --asset native --network $Network | Out-String).Trim()
Write-Host "TOKEN_ID = $TOKEN_ID"

# 4) Deploy contrato + constructor (__constructor(admin, token))
$deployOut = (stellar contract deploy `
  --wasm $WasmOpt `
  --source $AdminAlias `
  --network $Network `
  -- `
  --admin $ADMIN_ADDR `
  --token $TOKEN_ID | Out-String).Trim()

# Intenta extraer contract id (si tu CLI ya imprime el ID solo, tomá la salida completa)
$CONTRACT_ID = $deployOut
Write-Host "CONTRACT_ID = $CONTRACT_ID"

# 5) Configurar fee fijo (firma admin)
Run "stellar contract invoke --id $CONTRACT_ID --source $AdminAlias --network $Network -- set_admin_fee --fee $Fee"

# 6) Agregar auto (firma admin; owner es el dueño del auto)
Run "stellar contract invoke --id $CONTRACT_ID --source $AdminAlias --network $Network -- add_car --owner $OWNER_ADDR --price_per_day $PricePerDay"

# 7) Alquilar (depósito real = amount + fee). Firma renter.
$depositTotal = $Amount + $Fee
Write-Host "Deposit Total (amount + fee) = $depositTotal"
Write-Host "Saldo renter antes del alquiler:"
Run "stellar contract invoke --id $TOKEN_ID --source $RenterAlias --network $Network -- balance --id $RENTER_ADDR"
Write-Host "Saldo del contrato antes del alquiler:"
Run "stellar contract invoke --id $TOKEN_ID --source $AdminAlias --network $Network -- balance --id $CONTRACT_ID"
# Capturas previas para resumen
$balAdminBefore  = GetTokenBalance $AdminAlias  $ADMIN_ADDR
$balOwnerBefore  = GetTokenBalance $OwnerAlias  $OWNER_ADDR
$balRenterBefore = GetTokenBalance $RenterAlias $RENTER_ADDR
$balContractBefore = GetTokenBalance $AdminAlias $CONTRACT_ID
Run "stellar contract invoke --id $CONTRACT_ID --source $RenterAlias --network $Network -- rental --renter $RENTER_ADDR --owner $OWNER_ADDR --total_days_to_rent $TotalDays --amount $Amount"

# 8) Consultas útiles (opcional):
#   - Saldo del contrato en el token
Write-Host "Saldo del contrato (debería ser depositTotal ahora):"
Run "stellar contract invoke --id $TOKEN_ID --source $AdminAlias --network $Network -- balance --id $CONTRACT_ID"
# Captura post-rental
$balContractAfterRental = GetTokenBalance $AdminAlias $CONTRACT_ID
$balAdminAfterRental    = GetTokenBalance $AdminAlias $ADMIN_ADDR
$balOwnerAfterRental    = GetTokenBalance $OwnerAlias $OWNER_ADDR
$balRenterAfterRental   = GetTokenBalance $RenterAlias $RENTER_ADDR

# 9) Retiro admin (firma admin) - comisiones
Run "stellar contract invoke --id $CONTRACT_ID --source $AdminAlias --network $Network -- withdraw_admin"

Write-Host "Saldo del contrato después de retirar admin (debería restar el fee):"
Run "stellar contract invoke --id $TOKEN_ID --source $AdminAlias --network $Network -- balance --id $CONTRACT_ID"
# Capturas post-admin withdraw
$balAdminAfter   = GetTokenBalance $AdminAlias $ADMIN_ADDR
$balContractAfterAdmin = GetTokenBalance $AdminAlias $CONTRACT_ID

# 10) Devolver auto (firma renter)
Run "stellar contract invoke --id $CONTRACT_ID --source $RenterAlias --network $Network -- return_car --renter $RENTER_ADDR --owner $OWNER_ADDR"

# 11) Retiro owner (firma owner) - sólo si Available
Run "stellar contract invoke --id $CONTRACT_ID --source $OwnerAlias --network $Network -- withdraw_owner --owner $OWNER_ADDR"

Write-Host "Saldo del contrato después de retirar owner (debería quedar en 0):"
Run "stellar contract invoke --id $TOKEN_ID --source $AdminAlias --network $Network -- balance --id $CONTRACT_ID"
# Capturas finales
$balOwnerAfter   = GetTokenBalance $OwnerAlias $OWNER_ADDR
$balRenterAfter  = GetTokenBalance $RenterAlias $RENTER_ADDR
$balContractAfterOwner = GetTokenBalance $AdminAlias $CONTRACT_ID

# 12) (Opcional) Ver saldos individuales en el token
Write-Host "Saldo admin:"
Run "stellar contract invoke --id $TOKEN_ID --source $AdminAlias --network $Network -- balance --id $ADMIN_ADDR"
Write-Host "Saldo owner:"
Run "stellar contract invoke --id $TOKEN_ID --source $OwnerAlias --network $Network -- balance --id $OWNER_ADDR"
Write-Host "Saldo renter:"
Run "stellar contract invoke --id $TOKEN_ID --source $RenterAlias --network $Network -- balance --id $RENTER_ADDR"

# Tabla resumen
Write-Host ""
if ($VerboseReport) {
  Write-Host "Resumen de balances (stroops / XLM)" -ForegroundColor Yellow
  Write-Host ("{0,-10} {1,24} {2,24} {3,24} {4,24}" -f "Entidad","Antes rental","Después rental","Después admin","Después owner")

  function Row($label, [decimal]$b1, [decimal]$b2, [decimal]$b3, [decimal]$b4) {
    $s1 = "{0} / {1}" -f $b1, (ToXlm $b1)
    $s2 = "{0} / {1}" -f $b2, (ToXlm $b2)
    $s3 = "{0} / {1}" -f $b3, (ToXlm $b3)
    $s4 = "{0} / {1}" -f $b4, (ToXlm $b4)
    Write-Host ("{0,-10} {1,24} {2,24} {3,24} {4,24}" -f $label,$s1,$s2,$s3,$s4)
  }

  Row "Contrato" $balContractBefore $balContractAfterRental $balContractAfterAdmin $balContractAfterOwner
  Row "Admin"    $balAdminBefore   $balAdminBefore         $balAdminAfter       $balAdminAfter
  Row "Owner"    $balOwnerBefore   $balOwnerBefore         $balOwnerBefore      $balOwnerAfter
  Row "Renter"   $balRenterBefore  $balRenterBefore        $balRenterBefore     $balRenterAfter
}

# Tabla de variaciones (deltas)
Write-Host ""
if ($VerboseReport) {
  Write-Host "Variaciones (delta) en stroops / XLM" -ForegroundColor Yellow
  Write-Host ("{0,-10} {1,22} {2,22} {3,22} {4,22} {5,22} {6,22}" -f "Entidad","Exp Δrental","Act Δrental","Exp Δadmin","Act Δadmin","Exp Δowner","Act Δowner")

  function RowDelta($label, [decimal]$expR, [decimal]$actR, [decimal]$expA, [decimal]$actA, [decimal]$expO, [decimal]$actO) {
    function F([decimal]$v) { "{0} / {1}" -f $v, (ToXlm $v) }
    $s1 = F $expR; $s2 = F $actR; $s3 = F $expA; $s4 = F $actA; $s5 = F $expO; $s6 = F $actO
    Write-Host ("{0,-10} {1,22} {2,22} {3,22} {4,22} {5,22} {6,22}" -f $label,$s1,$s2,$s3,$s4,$s5,$s6)
  }

  # Esperado (sin fees de red)
  $expCR = [decimal]$depositTotal; $expCA = -[decimal]$Fee;   $expCO = -[decimal]$Amount
  $expAR = [decimal]0;            $expAA =  [decimal]$Fee;   $expAO =  [decimal]0
  $expOR = [decimal]0;            $expOA =  [decimal]0;      $expOO =  [decimal]$Amount
  $expRR = -[decimal]$depositTotal; $expRA = [decimal]0;     $expRO =  [decimal]0

  # Actual (incluye fees de transacciones)
  $actCR = $balContractAfterRental - $balContractBefore
  $actCA = $balContractAfterAdmin  - $balContractAfterRental
  $actCO = $balContractAfterOwner  - $balContractAfterAdmin

  $actAR = $balAdminAfterRental - $balAdminBefore
  $actAA = $balAdminAfter       - $balAdminAfterRental
  $actAO = [decimal]0

  $actOR = $balOwnerAfterRental - $balOwnerBefore
  $actOA = [decimal]0
  $actOO = $balOwnerAfter       - $balOwnerAfterRental

  $actRR = $balRenterAfterRental - $balRenterBefore
  $actRA = [decimal]0
  $actRO = $balRenterAfter       - $balRenterAfterRental

  RowDelta "Contrato" $expCR $actCR $expCA $actCA $expCO $actCO
  RowDelta "Admin"    $expAR $actAR $expAA $actAA $expAO $actAO
  RowDelta "Owner"    $expOR $actOR $expOA $actOA $expOO $actOO
  RowDelta "Renter"   $expRR $actRR $expRA $actRA $expRO $actRO
}

# Resumen simple y legible
Write-Host ""
Write-Host "Resumen simple" -ForegroundColor Yellow

function FX([decimal]$v) { "{0} ({1} XLM)" -f $v, (ToXlm $v) }

# Valores clave
$depositStr = FX([decimal]$depositTotal)
$feeStr     = FX([decimal]$Fee)
$amountStr  = FX([decimal]$Amount)

# Checks de consistencia
$okDeposit = ($balContractAfterRental - $balContractBefore - [decimal]$depositTotal) -eq 0
$okAdmin   = ($balContractAfterAdmin  - $balContractAfterRental + [decimal]$Fee) -eq 0
$okOwner   = ($balContractAfterOwner  - $balContractAfterAdmin  + [decimal]$Amount) -eq 0

# Renter (alquilar)
Write-Host ("Renter: antes {0} | después rental {1}" -f (FX $balRenterBefore), (FX $balRenterAfterRental))
Write-Host ("Depósito enviado: {0} | Δ contrato OK={1}" -f $depositStr, $okDeposit)

# Admin (retira fee)
Write-Host ("Admin: antes {0} | después retiro {1}" -f (FX $balAdminBefore), (FX $balAdminAfter))
Write-Host ("Fee recibido: {0} | Δ contrato OK={1}" -f $feeStr, $okAdmin)

# Owner (retira monto base)
Write-Host ("Owner: antes {0} | después retiro {1}" -f (FX $balOwnerBefore), (FX $balOwnerAfter))
Write-Host ("Pago recibido: {0} | Δ contrato OK={1}" -f $amountStr, $okOwner)

# Contrato (saldo agregado al resumen)
Write-Host ("Contrato: antes {0} | después rental {1} | después admin {2} | después owner {3}" -f (FX $balContractBefore), (FX $balContractAfterRental), (FX $balContractAfterAdmin), (FX $balContractAfterOwner))
