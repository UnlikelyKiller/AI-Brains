# Ingest-HermesVault.ps1 - Ingest all Hermes vault notes into AI-Brains
# Run from Windows PowerShell: .\\Ingest-HermesVault.ps1

$ErrorActionPreference = "Continue"

$VaultPath = "C:\\dev\\ai-brains\\vault.db"
$HermesDir = "C:\\Users\\RyanB\\Documents\\Hermes"
$ProjectId = "66dd77f4-57cc-528b-972d-7478dc58ea8d"
$HarnessId = "hermes-vault-ingest"
$SessionId = [Guid]::NewGuid().ToString()
$AiBrains = "C:\\Users\\RyanB\\.cargo\\bin\\ai-brains.exe"

Write-Host "AI-Brains Hermes Vault Ingestion"
Write-Host "Project: $ProjectId"
Write-Host "Session: $SessionId"
Write-Host "Vault:   $VaultPath"
Write-Host "Source:  $HermesDir"
Write-Host "=" * 60

$files = Get-ChildItem -Path $HermesDir -Filter "*.md" -Recurse | Sort-Object FullName
Write-Host "Found $($files.Count) markdown files"

$success = 0
$failed = 0

foreach ($file in $files) {
    $relPath = $file.FullName.Substring($HermesDir.Length + 1)
    $content = Get-Content -Path $file.FullName -Raw -Encoding UTF8
    
    if ($content.Trim().Length -eq 0) {
        Write-Host "  [skip] $relPath (empty)" -ForegroundColor Gray
        $success++
        continue
    }
    
    # Truncate if extremely large
    $maxLen = 50000
    if ($content.Length -gt $maxLen) {
        $content = $content.Substring(0, $maxLen) + "`n`n[...truncated for size...]"
    }
    
    $turnId = [Guid]::NewGuid().ToString()
    $memoryId = "hermes-vault/$relPath"
    
    # Metadata header
    $fullContent = @"
---
source: $memoryId
type: knowledge_base_note
project: Hermes
---
$content
"@
    
    $json = @{
        session_id = $SessionId
        project_id = $ProjectId
        harness_id = $HarnessId
        turn_id = $turnId
        role = "system"
        content = $fullContent
        privacy = "CloudOk"
    } | ConvertTo-Json -Compress
    
    try {
        $result = $json | & $AiBrains --vault-path $VaultPath ingest 2>&1
        $parsed = $result | ConvertFrom-Json
        if ($parsed.processed) {
            Write-Host "  [ok]   $relPath" -ForegroundColor Green
            $success++
        } else {
            Write-Host "  [fail] $relPath : $($parsed.message)" -ForegroundColor Red
            $failed++
        }
    } catch {
        Write-Host "  [fail] $relPath : $_" -ForegroundColor Red
        $failed++
    }
    
    # Small delay to avoid lock contention
    Start-Sleep -Milliseconds 50
}

Write-Host ""
Write-Host "=" * 60
Write-Host "Summary: $success succeeded, $failed failed"
Write-Host "Session: $SessionId"
Write-Host ""
Write-Host "Now running nightly intelligence sweep..."
& $AiBrains --vault-path $VaultPath nightly 2>&1
