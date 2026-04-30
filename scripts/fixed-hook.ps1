# AI-Brains Hook for Gemini CLI
# Handles Preflight (BeforeAgent/SessionStart) and Ingest (AfterAgent) lifecycle events.

$logPrefix = "[ai-brains-hook]"

# Read stdin
try {
    $stdin = [Console]::In.ReadToEnd()
    if (-not $stdin) {
        $stdin = $input | Out-String
    }
    $inputJson = $stdin | ConvertFrom-Json
} catch {
    Write-Error "$logPrefix Failed to parse JSON input: $_"
    @{ success = $true } | ConvertTo-Json -Compress
    exit 0
}

# Helper to load .env manually if needed
function Load-Env($path) {
    if (Test-Path $path) {
        Write-Error "$logPrefix Loading env from $path"
        $content = Get-Content $path -Raw
        $content -split "`r?`n" | ForEach-Object {
            if ($_ -match '^\s*([^#\s][^=]*)\s*=\s*(.*)$') {
                $name = $matches[1].Trim()
                $value = $matches[2].Trim().Trim('"').Trim("'")
                if (-not (Test-Path "Env:$name")) {
                    Set-Item "Env:$name" $value
                }
            }
        }
    }
}

# Ensure environment is loaded
if ($env:GEMINI_PROJECT_DIR) {
    Load-Env (Join-Path $env:GEMINI_PROJECT_DIR ".env")
}
Load-Env (Join-Path $HOME ".ai-brains\.env")

function Handle-Preflight($data) {
    Write-Error "$logPrefix Running preflight..."
    $preflightJson = ai-brains preflight --max-words 1500
    if ($LASTEXITCODE -ne 0) {
        Write-Error "$logPrefix Preflight failed with exit code $LASTEXITCODE"
        return @{ success = $true }
    }

    $preflight = $preflightJson | ConvertFrom-Json
    return @{
        success = $true
        hookSpecificOutput = @{
            additionalContext = $preflight.text
        }
    }
}

function Handle-Ingest($data) {
    Write-Error "$logPrefix Running ingest..."
    $payload = $data.payload
    $content = $payload.final_response
    if (-not $content) { $content = $payload.prompt_response }
    if (-not $content) { $content = $data.final_response }
    if (-not $content) { $content = $data.prompt_response }

    if (-not $content) {
        Write-Error "$logPrefix No content found in payload."
        return @{ success = $true; systemMessage = "No response content found to ingest." }
    }

    $ingestScript = $null
    if ($env:GEMINI_PROJECT_DIR) {
        $localScript = Join-Path $env:GEMINI_PROJECT_DIR ".agents\skills\ai-brains\scripts\ingest.ps1"
        if (Test-Path $localScript) { $ingestScript = $localScript }
    }

    if ($ingestScript) {
        Write-Error "$logPrefix Calling ingest script: $ingestScript"
        Push-Location $env:GEMINI_PROJECT_DIR
        try {
            # Redirect stdout to null to keep the hook output clean
            & $ingestScript -Content $content -Role "assistant" | Out-Null
        } finally {
            Pop-Location
        }
    } else {
        Write-Error "$logPrefix Falling back to direct CLI ingest."
        $harnessId = $env:AI_BRAINS_HARNESS_ID
        if (-not $harnessId) { $harnessId = "default-harness" }
        $projectId = $env:AI_BRAINS_PROJECT_ID
        $sessionId = $env:AI_BRAINS_SESSION_ID

        if ($projectId -and $sessionId) {
            $ingestPayload = @{
                session_id = $sessionId
                project_id = $projectId
                harness_id = $harnessId
                turn_id = [guid]::NewGuid().ToString()
                role = "assistant"
                content = $content
                privacy = "LocalOnly"
            } | ConvertTo-Json -Compress
            $ingestPayload | ai-brains ingest | Out-Null
        } else {
            Write-Error "$logPrefix Missing ProjectID ($projectId) or SessionID ($sessionId)"
        }
    }

    return @{
        success = $true
        systemMessage = "Memory captured successfully."
    }
}

$event = $inputJson.hook_event_name
if (-not $event) { $event = $inputJson.hook_type }
Write-Error "$logPrefix Event detected: $event"

switch ($event) {
    "SessionStart" { Handle-Preflight $inputJson | ConvertTo-Json -Compress }
    "BeforeAgent"  { Handle-Preflight $inputJson | ConvertTo-Json -Compress }
    "AfterAgent"   { Handle-Ingest $inputJson | ConvertTo-Json -Compress }
    default { @{ success = $true } | ConvertTo-Json -Compress }
}
