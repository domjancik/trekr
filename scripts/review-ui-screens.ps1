param(
    [string]$ScreenshotDir = "artifacts/screenshots",
    [string]$OutputFile = "artifacts/reviews/ui-findings.md",
    [string]$Model = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$screensRoot = Join-Path $repoRoot $ScreenshotDir
$outputPath = Join-Path $repoRoot $OutputFile
$promptPath = Join-Path $PSScriptRoot "ui-review-prompt.md"

if (-not (Test-Path $screensRoot)) {
    throw "Screenshot directory not found: $screensRoot"
}

$images = Get-ChildItem -Path $screensRoot -Filter *.png | Sort-Object Name
if ($images.Count -eq 0) {
    throw "No screenshots found in $screensRoot"
}

$outputDir = Split-Path -Parent $outputPath
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

$prompt = Get-Content $promptPath -Raw
$prompt += "`n`nScreenshots:`n"
foreach ($image in $images) {
    $prompt += "- $($image.FullName)`n"
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
if ($LASTEXITCODE -ne 0) {
    throw "codex review failed with exit code $LASTEXITCODE"
}

Write-Host "UI findings written to $outputPath"
