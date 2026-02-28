param(
    [string]$OutputDir = "artifacts/screenshots",
    [string]$FindingsFile = "artifacts/reviews/ui-findings.md",
    [string]$ArchiveRoot = "artifacts/archive",
    [string]$Model = "",
    [string]$StateMode = "demo",
    [string]$StateFile = ""
)

$ErrorActionPreference = "Stop"
$scriptRoot = $PSScriptRoot
$repoRoot = Split-Path -Parent $scriptRoot

& (Join-Path $scriptRoot "capture-ui-screens.ps1") -OutputDir $OutputDir -StateMode $StateMode -StateFile $StateFile
& (Join-Path $scriptRoot "review-ui-screens.ps1") -ScreenshotDir $OutputDir -OutputFile $FindingsFile -Model $Model

$commit = (& git -c safe.directory=$repoRoot rev-parse --short HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($commit)) {
    throw "failed to resolve current git commit for archive snapshot"
}

$archiveDir = Join-Path $repoRoot $ArchiveRoot
$commitArchiveDir = Join-Path $archiveDir $commit
$screensArchiveDir = Join-Path $commitArchiveDir "screenshots"
$reviewsArchiveDir = Join-Path $commitArchiveDir "reviews"
 $screensSourceDir = Join-Path $repoRoot $OutputDir

New-Item -ItemType Directory -Force -Path $screensArchiveDir | Out-Null
New-Item -ItemType Directory -Force -Path $reviewsArchiveDir | Out-Null

Copy-Item -Path (Join-Path $screensSourceDir "*") -Destination $screensArchiveDir -Recurse -Force
Copy-Item -Path (Join-Path $repoRoot $FindingsFile) -Destination (Join-Path $reviewsArchiveDir "ui-findings.md") -Force

Write-Host "Archived UI review snapshot for commit ${commit}:"
Write-Host " - screenshots: $screensArchiveDir"
Write-Host " - findings: $(Join-Path $reviewsArchiveDir 'ui-findings.md')"
