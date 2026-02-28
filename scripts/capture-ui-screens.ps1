param(
    [string]$OutputDir = "artifacts/screenshots"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$outputRoot = Join-Path $repoRoot $OutputDir
$binaryPath = Join-Path $repoRoot "target\debug\trekr.exe"

Get-Process trekr -ErrorAction SilentlyContinue | Stop-Process -Force

& cargo build | Out-Host
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
}

New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
Remove-Item -Path (Join-Path $outputRoot "*.png") -Force -ErrorAction SilentlyContinue
Remove-Item -Path (Join-Path $outputRoot "manifest.json") -Force -ErrorAction SilentlyContinue

$args = @(
    "--capture-ui",
    "--capture-dir", $outputRoot
)

& $binaryPath @args | Out-Host
if ($LASTEXITCODE -ne 0) {
    throw "trekr UI capture failed with exit code $LASTEXITCODE"
}

$captures = Get-ChildItem -Path $outputRoot -Filter *.png | Sort-Object Name
$captureCount = @($captures).Count
if ($captureCount -eq 0) {
    throw "trekr UI capture produced no screenshots in $outputRoot"
}

$manifest = foreach ($capture in $captures) {
    [pscustomobject]@{
        page = [System.IO.Path]::GetFileNameWithoutExtension($capture.Name)
        path = $capture.FullName
    }
}

$manifestPath = Join-Path $outputRoot "manifest.json"
$manifest | ConvertTo-Json | Set-Content -Encoding UTF8 $manifestPath

Write-Host "Captured renderer-level screenshots:"
$manifest | ForEach-Object { Write-Host " - $($_.page): $($_.path)" }
Write-Host "Manifest: $(Resolve-Path $manifestPath)"
