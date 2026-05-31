$ErrorActionPreference = "Stop"
Set-Location "C:\dev\AI-Brains"

Write-Host "Building ai-brains..." -ForegroundColor Cyan
cargo build --release -p ai-brains-cli

if ($LASTEXITCODE -eq 0) {
    Copy-Item "target\release\ai-brains.exe" "C:\Users\RyanB\.cargo\bin\ai-brains.exe" -Force
    Write-Host "BUILD SUCCESS" -ForegroundColor Green
    
    # Verify
    $ver = & "C:\Users\RyanB\.cargo\bin\ai-brains.exe" --version 2>$null
    Write-Host "Version: $ver"
} else {
    Write-Host "BUILD FAILED" -ForegroundColor Red
    exit 1
}
