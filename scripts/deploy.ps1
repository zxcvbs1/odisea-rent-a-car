param(
  [string]$EnvFile = ".env",
  [switch]$GenerateKeysIfMissing,
  [string]$OutEnv = ".env.contract",
  [string]$AdminSecret
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Rutas del repo y crate (usar target del crate)
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".." )).Path
$CrateDir = Join-Path $RepoRoot "contracts/rent-a-car"

function Load-DotEnv([string]$path) {
  $cfg = @{}
  if (-not (Test-Path $path)) { return $cfg }
  Get-Content $path | ForEach-Object {
    $line = $_.Trim()
    if ($line -eq "" -or $line.StartsWith("#")) { return }
    $idx = $line.IndexOf('=')
    if ($idx -lt 1) { return }
    $k = $line.Substring(0, $idx).Trim()
    $v = $line.Substring($idx + 1).Trim()
    $cfg[$k] = $v
  }
  return $cfg
}

function Run($cmd) {
  Write-Host ">> $cmd" -ForegroundColor Cyan
  iex $cmd
}

function OutText([string]$path, [string]$text) {
  [System.IO.File]::AppendAllText($path, $text)
}

function Update-EnvFile([string]$path, $pairs) {
  $existing = @()
  if (Test-Path $path) { $existing = Get-Content $path }
  $keys = @($pairs.Keys)
  # remove any existing lines for the keys we will set
  $filtered = $existing | Where-Object { $line = $_.Trim(); if ($line -eq '' -or $line.StartsWith('#')) { $true } else { $k = ($line.Split('=')[0]).Trim(); -not ($keys -contains $k) } }
  $updates = @()
  foreach ($k in $keys) { $updates += ("{0}={1}" -f $k, $pairs[$k]) }
  $out = @()
  $out += $filtered
  $out += $updates
  $out | Set-Content $path
}

function Remove-EnvKeys([string]$path, [string[]]$keys) {
  if (-not (Test-Path $path)) { return }
  $existing = Get-Content $path
  $filtered = $existing | Where-Object {
    $line = $_.Trim()
    if ($line -eq '' -or $line.StartsWith('#')) { $true }
    else {
      $k = ($line.Split('=')[0]).Trim()
      -not ($keys -contains $k)
    }
  }
  $filtered | Set-Content $path
}

function Ensure-Key([string]$alias) {
  try {
    $addr = (stellar keys address $alias | Out-String).Trim()
    if ($addr) { return $true }
  } catch {}
  if ($AdminSecret) {
    Run "stellar keys import $alias --secret-key $AdminSecret"
    return $true
  }
  if ($GenerateKeysIfMissing) {
    Run "stellar keys generate $alias"
    return $true
  }
  throw "Identity '$alias' not found. Import with 'stellar keys import $alias --secret-key <S...>' or run with -GenerateKeysIfMissing"
}

# Load config
$cfg = Load-DotEnv $EnvFile
$Network    = $cfg["NETWORK"];    if (-not $Network) { $Network = "testnet" }
$AdminAlias = $cfg["ADMIN_ALIAS"]; if (-not $AdminAlias) { $AdminAlias = "admin" }
$TokenIdCfg = $cfg["TOKEN_ID"]
$TokenAsset = $cfg["TOKEN_ASSET"]  # e.g. native or CODE:G... (issuer)

# Ensure admin identity
Ensure-Key $AdminAlias | Out-Null
$AdminAddr = (stellar keys address $AdminAlias | Out-String).Trim()

# Build wasm (usando manifest del crate)
$ManifestPath = Join-Path $CrateDir "Cargo.toml"
Run "cargo build --manifest-path `"$ManifestPath`" --target wasm32v1-none --release"

# Resolve wasm path (use env WASM_PATH or auto-detect first .wasm)
$WasmPath = $cfg["WASM_PATH"]
if (-not $WasmPath) {
  $dir = Join-Path $CrateDir "target/wasm32v1-none/release"
  if (-not (Test-Path $dir)) { throw "Expected '$dir' to exist after build" }
  $found = Get-ChildItem $dir -Filter *.wasm | Select-Object -First 1
  if (-not $found) { throw "No .wasm found under $dir. Ensure crate-type = 'cdylib' in Cargo.toml" }
  $WasmPath = $found.FullName
}

Run "stellar contract optimize --wasm `"$WasmPath`""

# Resolve token id
if ($TokenIdCfg) {
  $TokenId = $TokenIdCfg
} elseif ($TokenAsset) {
  $TokenId = (stellar contract asset id --asset $TokenAsset --network $Network | Out-String).Trim()
} else {
  # default to native if nothing provided
  $TokenId = (stellar contract asset id --asset native --network $Network | Out-String).Trim()
}

Write-Host "NETWORK      = $Network"
Write-Host "ADMIN_ALIAS  = $AdminAlias"
Write-Host "ADMIN_ADDR   = $AdminAddr"
Write-Host "TOKEN_ID     = $TokenId"
Write-Host "WASM         = $WasmPath"

# Deploy
$deployOut = (stellar contract deploy `
  --wasm $WasmPath.Replace('.wasm','.optimized.wasm') `
  --source $AdminAlias `
  --network $Network `
  -- `
  --admin $AdminAddr `
  --token $TokenId | Out-String).Trim()

$ContractId = $deployOut
Write-Host "CONTRACT_ID  = $ContractId" -ForegroundColor Green

# Save CONTRACT_ID to .env.contract
$outLines = @()
if (Test-Path $OutEnv) { $outLines += (Get-Content $OutEnv) }
$outLines = $outLines | Where-Object { -not ($_.StartsWith("CONTRACT_ID=")) }
$outLines += "CONTRACT_ID=$ContractId"
$outLines | Set-Content $OutEnv
Write-Host "Saved CONTRACT_ID to $OutEnv"

# Ensure .env contains all resolved values
$pubNet = if ($Network -ieq "testnet") { "TESTNET" } else { "MAINNET" }
$pass   = if ($Network -ieq "testnet") { "Test SDF Network ; September 2015" } else { "Public Global Stellar Network ; September 2015" }
$rpcUrl = if ($Network -ieq "testnet") { "https://soroban-testnet.stellar.org" } else { "https://soroban-rpc.stellar.org" }
$hzUrl  = if ($Network -ieq "testnet") { "https://horizon-testnet.stellar.org" } else { "https://horizon.stellar.org" }

$envPairs = @{
  "NETWORK"     = $Network
  "ADMIN_ALIAS" = $AdminAlias
  "ADMIN_ADDR"  = $AdminAddr
  "TOKEN_ID"    = $TokenId
  "CONTRACT_ID" = $ContractId
  "PUBLIC_STELLAR_NETWORK" = $pubNet
  "PUBLIC_STELLAR_NETWORK_PASSPHRASE" = $pass
  "PUBLIC_STELLAR_RPC_URL" = $rpcUrl
  "PUBLIC_STELLAR_HORIZON_URL" = $hzUrl
  }
if ($TokenAsset) { $envPairs["TOKEN_ASSET"] = $TokenAsset }
Update-EnvFile -path $EnvFile -pairs $envPairs
Remove-EnvKeys -path $EnvFile -keys @("WASM_PATH")
Write-Host "Updated $EnvFile with NETWORK, ADMIN_ALIAS, ADMIN_ADDR, TOKEN_ID, CONTRACT_ID, and public endpoints" -ForegroundColor Green

Write-Host "Done." -ForegroundColor Green
