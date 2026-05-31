$ErrorActionPreference = "Stop"

# Copy the ingest script from WSL to Windows
$wslScript = "/home/ryan/.hermes/scripts/ai-brains-ingest.py"
$winDir = "C:\Users\RyanB\.hermes\scripts"
$winScript = "$winDir\ai-brains-ingest.py"

# Create directory if needed
if (-not (Test-Path $winDir)) {
    New-Item -ItemType Directory -Path $winDir -Force | Out-Null
}

# Read from WSL and write to Windows
$wslPath = "\\wsl$\Ubuntu\home\ryan\.hermes\scripts\ai-brains-ingest.py"
if (Test-Path $wslPath) {
    Copy-Item $wslPath $winScript -Force
    Write-Host "✅ Copied ai-brains-ingest.py to $winScript" -ForegroundColor Green
} else {
    Write-Host "❌ WSL script not found at $wslPath" -ForegroundColor Red
}
