# AI-Brains CI Gate Verification Script (T71)
# Checks tool presence and versions, then runs the full CI gate.
# Usage: .\scripts\dev-check.ps1 [--check-only]
#   --check-only  Only verify tool presence; skip running the gate.

param([switch]$CheckOnly)

$ErrorActionPreference = "Stop"

# ---------------------------------------------------------------------------
# Required tool versions (update when intentionally upgrading)
# ---------------------------------------------------------------------------
$Required = @{
    "cargo-nextest" = @{ MinVersion = "0.9.137"; InstallCmd = "cargo install cargo-nextest --locked" }
    "cargo-deny"    = @{ MinVersion = "0.19.4";  InstallCmd = "cargo install cargo-deny --locked" }
    "cargo-audit"   = @{ MinVersion = "0.22.1";  InstallCmd = "cargo install cargo-audit --locked" }
}

function Get-ToolVersion([string]$Name) {
    try {
        $raw = & $Name --version 2>&1 | Select-Object -First 1
        if ($raw -match "(\d+\.\d+\.\d+)") { return $Matches[1] }
    } catch {}
    return $null
}

function Compare-Versions([string]$Installed, [string]$Min) {
    $i = [System.Version]$Installed
    $m = [System.Version]$Min
    return $i -ge $m
}

# ---------------------------------------------------------------------------
# Preflight: verify tools are present and meet minimum versions
# ---------------------------------------------------------------------------
Write-Host "=== AI-Brains CI Gate — Tool Preflight ===" -ForegroundColor Cyan
$allOk = $true

foreach ($tool in $Required.Keys) {
    $info    = $Required[$tool]
    $version = Get-ToolVersion $tool
    if (-not $version) {
        Write-Host "  [MISSING] $tool — install with: $($info.InstallCmd)" -ForegroundColor Red
        $allOk = $false
    } elseif (-not (Compare-Versions $version $info.MinVersion)) {
        Write-Host "  [OUTDATED] $tool $version (need >= $($info.MinVersion)) — upgrade: $($info.InstallCmd)" -ForegroundColor Yellow
        $allOk = $false
    } else {
        Write-Host "  [OK] $tool $version" -ForegroundColor Green
    }
}

if (-not $allOk) {
    Write-Host "`n[FAIL] One or more tools are missing or outdated. Install them and re-run." -ForegroundColor Red
    exit 1
}

Write-Host ""

if ($CheckOnly) {
    Write-Host "[OK] All tools present. Skipping gate (--check-only)." -ForegroundColor Green
    exit 0
}

# ---------------------------------------------------------------------------
# Run the full CI gate
# ---------------------------------------------------------------------------
function Run-Step([string]$Label, [scriptblock]$Block) {
    Write-Host "--- $Label ---" -ForegroundColor Cyan
    & $Block
    if ($LASTEXITCODE -ne 0) {
        Write-Host "$Label FAILED" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    Write-Host ""
}

Run-Step "cargo fmt --check" { cargo fmt --check }
Run-Step "cargo clippy"      { cargo clippy --workspace --all-targets -- -D warnings }
Run-Step "cargo nextest"     { cargo nextest run --workspace }
Run-Step "cargo deny check"  { cargo deny check }
Run-Step "cargo audit"       { cargo audit }

Write-Host "[SUCCESS] CI Gate passed!" -ForegroundColor Green
