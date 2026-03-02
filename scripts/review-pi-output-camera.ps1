[CmdletBinding()]
param(
    [string]$CaptureDir = "artifacts/camera-debug",
    [string]$OutputFile = "artifacts/camera-debug/findings.md",
    [string]$Model = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$captureRoot = Join-Path $repoRoot $CaptureDir
$outputPath = Join-Path $repoRoot $OutputFile
$promptPath = Join-Path $PSScriptRoot "pi-output-camera-review-prompt.md"
$statusPath = Join-Path $captureRoot "pi-status.txt"

if (-not (Test-Path $captureRoot)) {
    throw "Capture directory not found: $captureRoot"
}

$images = @(
    Get-ChildItem -Path $captureRoot -Filter *.png -File -ErrorAction SilentlyContinue
    Get-ChildItem -Path $captureRoot -Filter *.jpg -File -ErrorAction SilentlyContinue
    Get-ChildItem -Path $captureRoot -Filter *.jpeg -File -ErrorAction SilentlyContinue
) | Sort-Object LastWriteTimeUtc -Descending
if ($images.Count -eq 0) {
    throw "No captured camera images found in $captureRoot"
}

$outputDir = Split-Path -Parent $outputPath
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

$prompt = Get-Content $promptPath -Raw
$prompt += "`n`nCamera captures:`n"
foreach ($image in $images) {
    $prompt += "- $($image.FullName)`n"
}

if (Test-Path $statusPath) {
    $prompt += [Environment]::NewLine + [Environment]::NewLine + "Pi status:" + [Environment]::NewLine + '```text' + [Environment]::NewLine
    $prompt += (Get-Content $statusPath -Raw)
    $prompt += [Environment]::NewLine + '```'
}

$args = @(
    "exec",
    "-C", $repoRoot,
    "--sandbox", "read-only",
    "-o", $outputPath,
    "-"
)

if ($Model -ne "") {
    $args += @("-m", $Model)
}

foreach ($image in $images) {
    $args += @("-i", $image.FullName)
}

$prompt | & codex @args
if ($LASTEXITCODE -ne 0 -and -not (Test-Path $outputPath)) {
    throw "codex camera review failed with exit code $LASTEXITCODE"
}

Write-Host "Camera review written to $outputPath"
