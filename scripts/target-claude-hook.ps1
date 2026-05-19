# AI-Brains Hook for Claude Code
# Handles SessionStart, Stop, SessionEnd, and PreCompact events.

# Initialize UTF-8 encoding (BOM-less) for standard streams and file I/O
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
$OutputEncoding = [Console]::InputEncoding = [Console]::OutputEncoding = $utf8NoBom

$logPrefix = '[ai-brains-claude]'

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
            if ($block.type -eq 'text' -and $block.text) {
                $textBlocks += $block.text
            } elseif ($block.type -eq 'output_text' -and $block.text) {
                $textBlocks += $block.text
            }
        }
        if ($textBlocks.Count -gt 0) { return ($textBlocks -join "`n") }
    }

    return $null
}

function Get-AssistantMessagesFromTranscript($transcriptPath, $tail, $limit) {
    if (-not $transcriptPath -or -not (Test-Path -LiteralPath $transcriptPath)) {
        return @()
    }

    $lines = @(Get-Content -LiteralPath $transcriptPath -Tail $tail)
    $messages = @()
    for ($i = $lines.Count - 1; $i -ge 0; $i--) {
        try {
            $entry = $lines[$i] | ConvertFrom-Json
            $isAssistant = $entry.role -eq 'assistant' -or $entry.type -eq 'assistant'
            if (-not $isAssistant) { continue }

            $text = Get-TextFromContent $entry.content
            if ($text) {
                $messages += $text
                if ($messages.Count -ge $limit) { break }
            }
        } catch { }
    }

    return $messages
}

function De-Noise($content) {
    if (-not $content) { return $null }

    $lines = $content -split "`r?`n"
    $filteredLines = [System.Collections.ArrayList]::new()
    $inCodeBlock = $false
    $currentBlock = [System.Collections.ArrayList]::new()

    foreach ($line in $lines) {
        if ($line -match '^```') {
            if ($inCodeBlock) {
                if ($currentBlock.Count -le 10) {
                    $filteredLines.Add('```') | Out-Null
                    foreach ($blockLine in $currentBlock) {
                        $filteredLines.Add($blockLine) | Out-Null
                    }
                    $filteredLines.Add('```') | Out-Null
                } else {
                    $filteredLines.Add('```... [Long block stripped] ...```') | Out-Null
                }
                $currentBlock = [System.Collections.ArrayList]::new()
                $inCodeBlock = $false
            } else {
                $inCodeBlock = $true
            }
            continue
        }

        if ($inCodeBlock) {
            $currentBlock.Add($line) | Out-Null
        } else {
            $filteredLines.Add($line) | Out-Null
        }
    }

    return ($filteredLines -join "`n")
}

function Invoke-Ingest($rawContent, $inputJson, $projectDir, $label) {
    if (-not $rawContent) {
        Write-Log "$label no assistant content found"
        return
    }

    $content = De-Noise $rawContent

    if ($projectDir) {
        $localScript = Join-Path $projectDir '.agents\skills\ai-brains\scripts\ingest.ps1'
        if (Test-Path -LiteralPath $localScript) {
            Write-Log "$label calling local ingest script"
            Push-Location $projectDir
            try {
                & $localScript -Content $content -Role 'assistant' | Out-Null
            } finally {
                Pop-Location
            }
            return
        }
    }

    Write-Log "$label falling back to direct CLI ingest"
    $harnessId = if ($env:AI_BRAINS_HARNESS_ID) { $env:AI_BRAINS_HARNESS_ID } else { 'claude-code' }
    $sessionId = if ($env:AI_BRAINS_SESSION_ID) { $env:AI_BRAINS_SESSION_ID } else { $inputJson.session_id }

    if (-not $env:AI_BRAINS_PROJECT_ID -or -not $sessionId) {
        Write-Log "$label missing project_id or session_id for direct ingest"
        return
    }

    $ingestPayload = @{
        session_id = $sessionId
        project_id = $env:AI_BRAINS_PROJECT_ID
        harness_id = $harnessId
        turn_id = [guid]::NewGuid().ToString()
        role = 'assistant'
        content = $content
        privacy = 'LocalOnly'
    } | ConvertTo-Json -Compress

    $tempFile = [System.IO.Path]::GetTempFileName()
    try {
        [System.IO.File]::WriteAllText($tempFile, $ingestPayload, $utf8NoBom)
        Get-Content -LiteralPath $tempFile -Raw | ai-brains ingest 2>$null | Out-Null
    } finally {
        if (Test-Path -LiteralPath $tempFile) { Remove-Item -LiteralPath $tempFile -Force }
    }
}

function Export-ClaudeEnv {
    if (-not $env:CLAUDE_ENV_FILE) { return }

    $envLines = [System.Collections.ArrayList]::new()
    if ($env:AI_BRAINS_PROJECT_ID) { $envLines.Add("export AI_BRAINS_PROJECT_ID=`"$($env:AI_BRAINS_PROJECT_ID)`"") | Out-Null }
    if ($env:AI_BRAINS_SESSION_ID) { $envLines.Add("export AI_BRAINS_SESSION_ID=`"$($env:AI_BRAINS_SESSION_ID)`"") | Out-Null }
    if ($env:AI_BRAINS_HARNESS_ID) { $envLines.Add("export AI_BRAINS_HARNESS_ID=`"$($env:AI_BRAINS_HARNESS_ID)`"") | Out-Null }

    if ($envLines.Count -gt 0) {
        [System.IO.File]::AppendAllLines($env:CLAUDE_ENV_FILE, $envLines, $utf8NoBom)
    }
}

function Invoke-Preflight {
    Write-Log 'Running preflight'
    $preflightRaw = ai-brains preflight --max-words 1500 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Preflight failed (exit $LASTEXITCODE)"
        Write-HookResponse @{ continue = $true }
        return
    }

    $preflightText = ($preflightRaw -join "`n")
    try {
        $preflightJson = $preflightRaw | ConvertFrom-Json
        if ($preflightJson.text) { $preflightText = $preflightJson.text }
    } catch { }

    Export-ClaudeEnv
    Write-HookResponse @{
        continue = $true
        hookSpecificOutput = @{
            hookEventName = 'SessionStart'
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
if (-not $projectDir) { $projectDir = $env:CLAUDE_PROJECT_DIR }
if (-not $projectDir) { $projectDir = $PWD.Path }

if ($projectDir) {
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
Write-Log "Event: $event | CWD: $projectDir"

switch ($event) {
    'SessionStart' {
        Invoke-Preflight
    }

    'Stop' {
        try {
            $messages = @(Get-AssistantMessagesFromTranscript $inputJson.transcript_path 50 1)
            if ($messages.Count -gt 0) {
                Invoke-Ingest $messages[0] $inputJson $projectDir 'Stop:'
                Write-Log 'Stop: ingest complete'
            } else {
                Write-Log 'Stop: no transcript assistant content found'
            }
        } catch {
            Write-Log "Stop: ingest failed: $_"
        }

        Write-HookResponse @{ continue = $true }
    }

    'SessionEnd' {
        try {
            $messages = @(Get-AssistantMessagesFromTranscript $inputJson.transcript_path 100 1)
            if ($messages.Count -gt 0) {
                Invoke-Ingest $messages[0] $inputJson $projectDir 'SessionEnd:'
                Write-Log 'SessionEnd: ingest complete'
            } else {
                Write-Log 'SessionEnd: no transcript assistant content found'
            }
        } catch {
            Write-Log "SessionEnd: ingest failed: $_"
        }

        Write-HookResponse @{ continue = $true }
    }

    'PreCompact' {
        try {
            $messages = @(Get-AssistantMessagesFromTranscript $inputJson.transcript_path 200 3)
            if ($messages.Count -gt 0) {
                $combinedContent = $messages -join "`n---`n"
                Invoke-Ingest $combinedContent $inputJson $projectDir 'PreCompact:'
                $msgCount = $messages.Count
                Write-Log "PreCompact: ingest complete ($msgCount turns captured)"
            } else {
                Write-Log 'PreCompact: no transcript assistant content found'
            }
        } catch {
            Write-Log "PreCompact: ingest failed: $_"
        }

        Write-HookResponse @{ continue = $true }
    }

    default {
        Write-HookResponse @{ continue = $true }
    }
}