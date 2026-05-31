# Build-AIBrains.ps1 - Build AI-Brains Windows binary with all features
# Run from C:\dev\AI-Brains

$ErrorActionPreference = "Stop"
$RepoPath = "C:\\dev\\AI-Brains"
$OutputBin = "C:\\Users\\RyanB\\.cargo\\bin\\ai-brains.exe"

Write-Host "Building AI-Brains Windows binary..." -ForegroundColor Cyan
Write-Host "Repo:    $RepoPath"
Write-Host "Output:  $OutputBin"
Write-Host "=" * 60

Set-Location $RepoPath

# Verify cargo is available
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    Write-Error "cargo not found in PATH. Is Rust installed?"
    exit 1
}

Write-Host "Rust toolchain: $(cargo --version)"

# Build release binary
cargo build --release -p ai-brains-cli
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed with exit code $LASTEXITCODE"
    exit 1
}

# Copy to cargo bin directory
$builtBin = "$RepoPath\\target\\release\\ai-brains.exe"
if (Test-Path $builtBin) {
    Copy-Item $builtBin $OutputBin -Force
    Write-Host "✅ Built and installed to $OutputBin" -ForegroundColor Green
} else {
    # Check if it's named ai-brains-new.exe
    $builtBin = "$RepoPath\\target\\release\\ai-brains-new.exe"
    if (Test-Path $builtBin) {
        Copy-Item $builtBin $OutputBin -Force
        Write-Host "✅ Built and installed to $OutputBin (from ai-brains-new.exe)" -ForegroundColor Green
    } else {
        Write-Error "Build output not found at expected paths"
        exit 1
    }
}

# Verify the binary works
Write-Host ""
Write-Host "Verifying build..."
& $OutputBin --version 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Binary responds to --version" -ForegroundColor Green
} else {
    Write-Warning "Binary may have issues"
}

Write-Host ""
Write-Host "Done! The binary now supports:"
Write-Host "  --semantic flag for embedding search"
Write-Host "  --global flag for cross-project preflight"
Write-Host "  All current codebase features"
