# AI-Brains Hook for Gemini CLI
# Handles Preflight (BeforeAgent/SessionStart) and Ingest (AfterAgent) lifecycle events.

# Initialize UTF-8 encoding (BOM-less) for standard streams and file I/O
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
$OutputEncoding = [Console]::InputEncoding = [Console]::OutputEncoding = $utf8NoBom

$logPrefix = "[ai-brains-gemini]"

function Write-Log($message) {
    [Console]::Error.WriteLine("$logPrefix $message")
}

function Write-HookResponse($response) {
    $response | ConvertTo-Json -Depth 10 -Compress
}

function Load-Env($path) {
    if (Test-Path -LiteralPath $path) {
        Write-Log "Loading env from $path"
        $content = Get-Content -LiteralPath $path -Raw
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

function Get-AssistantContent($data) {
    $payload = $data.payload

    if ($payload.final_response) { return $payload.final_response }
    if ($payload.prompt_response) { return $payload.prompt_response }
    if ($data.final_response) { return $data.final_response }
    if ($data.prompt_response) { return $data.prompt_response }

    return $null
}

function Invoke-Preflight($hookEventName) {
    Write-Log "Running preflight"
    $preflightRaw = ai-brains preflight --max-words 1500 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Preflight failed (exit $LASTEXITCODE)"
        Write-HookResponse @{ success = $true }
        return
    }

    $preflightText = ($preflightRaw -join "`n")
    try {
        $preflightJson = $preflightRaw | ConvertFrom-Json
        if ($preflightJson.text) { $preflightText = $preflightJson.text }
    } catch { }

    Write-HookResponse @{
        success = $true
        hookSpecificOutput = @{
            hookEventName = $hookEventName
            additionalContext = $preflightText
        }
    }
}

function De-Noise($content) {
    if (-not $content) { return $null }
    
    # Heuristic: Strip long markdown code blocks (usually test logs or file contents)
    # We keep short ones (under 10 lines) as they might be important snippets.
    $lines = $content -split "`r?`n"
    $filteredLines = @()
    $inCodeBlock = $false
    $currentBlock = @()

    foreach ($line in $lines) {
        if ($line -match '^```') {
            if ($inCodeBlock) {
                # End of block
                if ($currentBlock.Count -le 10) {
                    $filteredLines += "```"
                    $filteredLines += $currentBlock
                    $filteredLines += "```"
                } else {
                    $filteredLines += "```... [Long block stripped] ...```"
                }
                $currentBlock = @()
                $inCodeBlock = $false
            } else {
                # Start of block
                $inCodeBlock = $true
            }
            continue
        }

        if ($inCodeBlock) {
            $currentBlock += $line
        } else {
            $filteredLines += $line
        }
    }

    return ($filteredLines -join "`n")
}

function Invoke-Ingest($data) {
    Write-Log "Running ingest"
    $rawContent = Get-AssistantContent $data
    if (-not $rawContent) {
        Write-Log "No response content found to ingest"
        Write-HookResponse @{ success = $true; systemMessage = "No response content found to ingest." }
        return
    }

    $content = De-Noise $rawContent

    $projectDir = $env:GEMINI_PROJECT_DIR
    $ingestScript = $null
    if ($projectDir) {
        $localScript = Join-Path $projectDir ".agents\skills\ai-brains\scripts\ingest.ps1"
        if (Test-Path -LiteralPath $localScript) { $ingestScript = $localScript }
    }

    if ($ingestScript) {
        Write-Log "Calling local ingest script"
        Push-Location $projectDir
        try {
            & $ingestScript -Content $content -Role "assistant" | Out-Null
        } finally {
            Pop-Location
        }
    } else {
        Write-Log "Falling back to direct CLI ingest"
        $harnessId = if ($env:AI_BRAINS_HARNESS_ID) { $env:AI_BRAINS_HARNESS_ID } else { "gemini-cli" }
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

            $tempFile = [System.IO.Path]::GetTempFileName()
            try {
                [System.IO.File]::WriteAllText($tempFile, $ingestPayload, $utf8NoBom)
                Get-Content -LiteralPath $tempFile -Raw | ai-brains ingest 2>$null | Out-Null
            } finally {
                if (Test-Path -LiteralPath $tempFile) { Remove-Item -LiteralPath $tempFile -Force }
            }
        } else {
            Write-Log "Missing project_id or session_id for direct ingest"
        }
    }

    Write-HookResponse @{ success = $true; systemMessage = "Memory captured successfully." }
}

try {
    $stdin = [Console]::In.ReadToEnd()
    if (-not $stdin) { $stdin = $input | Out-String }
    $inputJson = $stdin | ConvertFrom-Json
} catch {
    Write-Log "Failed to parse JSON input: $_"
    Write-HookResponse @{ success = $true }
    exit 0
}

if ($env:GEMINI_PROJECT_DIR) {
    $projectDir = $env:GEMINI_PROJECT_DIR
    $projectEnv = Join-Path $projectDir '.env'
    if (Test-Path -LiteralPath $projectEnv) {
        Load-Env $projectEnv
    } else {
        # New Repository: Clear project-specific IDs to prevent leakage from the shell environment
        Remove-Item Env:AI_BRAINS_PROJECT_ID -ErrorAction SilentlyContinue
        Remove-Item Env:AI_BRAINS_SESSION_ID -ErrorAction SilentlyContinue
    }
}
Load-Env (Join-Path $HOME ".ai-brains\.env")

$event = $inputJson.hook_event_name
if (-not $event) { $event = $inputJson.hook_type }
Write-Log "Event detected: $event"

switch ($event) {
    "SessionStart" { Invoke-Preflight "SessionStart" }
    "BeforeAgent" { Invoke-Preflight "BeforeAgent" }
    "AfterAgent" { Invoke-Ingest $inputJson }
    default { Write-HookResponse @{ success = $true } }
}
