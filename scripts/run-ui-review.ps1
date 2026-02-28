param(
    [string]$OutputDir = "artifacts/screenshots",
    [string]$FindingsFile = "artifacts/reviews/ui-findings.md",
    [string]$Model = ""
)

$ErrorActionPreference = "Stop"
$scriptRoot = $PSScriptRoot

& (Join-Path $scriptRoot "capture-ui-screens.ps1") -OutputDir $OutputDir
& (Join-Path $scriptRoot "review-ui-screens.ps1") -ScreenshotDir $OutputDir -OutputFile $FindingsFile -Model $Model
