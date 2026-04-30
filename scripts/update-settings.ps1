$config = Get-Content C:\Users\RyanB\.gemini\settings.json | ConvertFrom-Json
$hook = @{
    matcher = "*"
    hooks = @(
        @{
            name = "ai-brains-preflight"
            type = "command"
            command = "powershell `"C:\Users\RyanB\.ai-brains\scripts\target-gemini-hook.ps1`""
        }
    )
}

# Update or add SessionStart
if (-not $config.hooks.SessionStart) {
    $config.hooks | Add-Member -MemberType NoteProperty -Name "SessionStart" -Value @($hook)
} else {
    $config.hooks.SessionStart = @($hook)
}

# Update or add BeforeAgent
if (-not $config.hooks.BeforeAgent) {
    $config.hooks | Add-Member -MemberType NoteProperty -Name "BeforeAgent" -Value @($hook)
} else {
    $config.hooks.BeforeAgent = @($hook)
}

# Update or add AfterAgent (use a different name if desired, but same script)
if (-not $config.hooks.AfterAgent) {
    $config.hooks | Add-Member -MemberType NoteProperty -Name "AfterAgent" -Value @($hook)
} else {
    $config.hooks.AfterAgent = @($hook)
}

$config | ConvertTo-Json -Depth 10 | Set-Content C:\Users\RyanB\.gemini\settings.json
