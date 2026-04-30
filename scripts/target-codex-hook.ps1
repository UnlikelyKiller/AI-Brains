# AI-Brains Hook for Codex CLI
# Handles SessionStart, UserPromptSubmit, and Stop events.

$logPrefix = "[ai-brains-codex]"

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

function Get-TextFromContent($content) {
    if (-not $content) { return $null }

    if ($content -is [string]) { return $content }

    if ($content -is [array]) {
        $textBlocks = @()
        foreach ($block in $content) {
            if ($block.type -eq "text" -and $block.text) {
                $textBlocks += $block.text
            } elseif ($block.type -eq "output_text" -and $block.text) {
                $textBlocks += $block.text
            }
        }
        if ($textBlocks.Count -gt 0) { return ($textBlocks -join "`n") }
    }

    return $null
}

function Get-LastAssistantMessageFromTranscript($transcriptPath) {
    if (-not $transcriptPath -or -not (Test-Path -LiteralPath $transcriptPath)) {
        return $null
    }

    $lines = Get-Content -LiteralPath $transcriptPath -Tail 200
    for ($i = $lines.Count - 1; $i -ge 0; $i--) {
        try {
            $entry = $lines[$i] | ConvertFrom-Json

            if ($entry.type -eq "event_msg" -and $entry.payload.type -eq "agent_message" -and $entry.payload.message) {
                return $entry.payload.message
            }

            if ($entry.type -eq "response_item" -and $entry.payload.type -eq "message" -and $entry.payload.role -eq "assistant") {
                $text = Get-TextFromContent $entry.payload.content
                if ($text) { return $text }
            }

            if ($entry.role -eq "assistant" -or $entry.type -eq "assistant") {
                $text = Get-TextFromContent $entry.content
                if ($text) { return $text }
            }
        } catch { }
    }

    return $null
}

function Invoke-Ingest($content, $inputJson, $projectDir, $role) {
    if (-not $content) {
        Write-Log "No $role content found for ingest"
        return
    }

    $localScript = $null
    if ($projectDir) {
        $candidate = Join-Path $projectDir ".agents\skills\ai-brains\scripts\ingest.ps1"
        if (Test-Path -LiteralPath $candidate) { $localScript = $candidate }
    }

    if ($localScript) {
        Write-Log "Calling local ingest script"
        Push-Location $projectDir
        try {
            & $localScript -Content $content -Role $role | Out-Null
        } finally {
            Pop-Location
        }
        return
    }

    Write-Log "Falling back to direct CLI ingest"
    $harnessId = if ($env:AI_BRAINS_HARNESS_ID) { $env:AI_BRAINS_HARNESS_ID } else { "codex-cli" }
    $sessionId = if ($env:AI_BRAINS_SESSION_ID) { $env:AI_BRAINS_SESSION_ID } else { $inputJson.session_id }

    if (-not $env:AI_BRAINS_PROJECT_ID -or -not $sessionId) {
        Write-Log "Missing project_id or session_id for direct ingest"
        return
    }

    $ingestPayload = @{
        session_id = $sessionId
        project_id = $env:AI_BRAINS_PROJECT_ID
        harness_id = $harnessId
        turn_id = if ($inputJson.turn_id) { $inputJson.turn_id } else { [guid]::NewGuid().ToString() }
        role = $role
        content = $content
        privacy = "LocalOnly"
    } | ConvertTo-Json -Compress

    $tempFile = [System.IO.Path]::GetTempFileName()
    try {
        $ingestPayload | Out-File -FilePath $tempFile -Encoding utf8
        Get-Content -LiteralPath $tempFile -Raw | ai-brains ingest 2>$null | Out-Null
    } finally {
        if (Test-Path -LiteralPath $tempFile) { Remove-Item -LiteralPath $tempFile -Force }
    }
}

function Get-PromptFromInput($inputJson) {
    if ($inputJson.prompt) {
        $text = Get-TextFromContent $inputJson.prompt
        if ($text) { return $text }
    }

    if ($inputJson.user_prompt) {
        $text = Get-TextFromContent $inputJson.user_prompt
        if ($text) { return $text }
    }

    if ($inputJson.message) {
        $text = Get-TextFromContent $inputJson.message
        if ($text) { return $text }
    }

    return $null
}

function Invoke-Preflight($hookEventName) {
    Write-Log "Running preflight"
    $preflightRaw = ai-brains preflight --max-words 1500 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Preflight failed (exit $LASTEXITCODE)"
        Write-HookResponse @{ continue = $true }
        return
    }

    $preflightText = $preflightRaw
    try {
        $preflightJson = $preflightRaw | ConvertFrom-Json
        if ($preflightJson.text) { $preflightText = $preflightJson.text }
    } catch { }

    Write-HookResponse @{
        continue = $true
        hookSpecificOutput = @{
            hookEventName = $hookEventName
            additionalContext = $preflightText
        }
    }
}

try {
    $stdin = [Console]::In.ReadToEnd()
    if (-not $stdin) { $stdin = $input | Out-String }
    $inputJson = $stdin | ConvertFrom-Json
} catch {
    Write-HookResponse @{ continue = $true }
    exit 0
}

$projectDir = $inputJson.cwd
if (-not $projectDir) { $projectDir = $env:CODEX_CWD }
if (-not $projectDir) { $projectDir = $PWD.Path }

if ($projectDir) { Load-Env (Join-Path $projectDir ".env") }
Load-Env (Join-Path $HOME ".ai-brains\.env")

$event = $inputJson.hook_event_name
Write-Log "Event: $event | CWD: $projectDir"

switch ($event) {
    "SessionStart" {
        Invoke-Preflight "SessionStart"
    }

    "UserPromptSubmit" {
        try {
            $prompt = Get-PromptFromInput $inputJson
            Invoke-Ingest $prompt $inputJson $projectDir "user"
            Write-Log "UserPromptSubmit ingest complete"
        } catch {
            Write-Log "UserPromptSubmit ingest failed: $_"
        }

        Write-HookResponse @{ continue = $true }
    }

    "Stop" {
        try {
            $content = $inputJson.last_assistant_message
            if (-not $content) {
                $content = Get-LastAssistantMessageFromTranscript $inputJson.transcript_path
            }

            Invoke-Ingest $content $inputJson $projectDir "assistant"
            Write-Log "Stop ingest complete"
        } catch {
            Write-Log "Stop ingest failed: $_"
        }

        Write-HookResponse @{ continue = $true }
    }

    default {
        Write-HookResponse @{ continue = $true }
    }
}
