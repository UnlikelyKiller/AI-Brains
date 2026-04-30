$targetDir = "C:\Users\RyanB\.local\bin"
$registryPath = "HKCU:\Environment"
$currentPath = (Get-ItemProperty -Path $registryPath -Name "Path").Path

if ($currentPath -split ";" -contains $targetDir) {
    Write-Host "Path already contains $targetDir"
} else {
    $newPath = "$currentPath;$targetDir".Replace(";;", ";")
    Set-ItemProperty -Path $registryPath -Name "Path" -Value $newPath
    Write-Host "Successfully added $targetDir to User PATH."
    # Broadcast change to other windows (optional but nice)
    # This part is hard in pure PS without P/Invoke, but for a dev session, a restart is standard.
}
