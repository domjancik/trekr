[CmdletBinding()]
param(
    [string]$ConfigPath = ".\scripts\pi-camera-debug.local.psd1",
    [string]$CaptureDir = "artifacts/camera-debug",
    [string]$FindingsFile = "artifacts/camera-debug/findings.md",
    [string]$ArchiveRoot = "artifacts/archive",
    [string]$Model = "",
    [switch]$SkipPiStatus
)

$ErrorActionPreference = "Stop"
$scriptRoot = $PSScriptRoot
$repoRoot = Split-Path -Parent $scriptRoot

& (Join-Path $scriptRoot "capture-pi-output-camera.ps1") -ConfigPath $ConfigPath -OutputDir $CaptureDir -SkipPiStatus:$SkipPiStatus
& (Join-Path $scriptRoot "review-pi-output-camera.ps1") -CaptureDir $CaptureDir -OutputFile $FindingsFile -Model $Model

$commit = (& git -c safe.directory=$repoRoot rev-parse --short HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($commit)) {
    throw "failed to resolve current git commit for archive snapshot"
}

$archiveDir = Join-Path $repoRoot $ArchiveRoot
$commitArchiveDir = Join-Path $archiveDir $commit
$cameraArchiveDir = Join-Path $commitArchiveDir "camera-debug"
New-Item -ItemType Directory -Force -Path $cameraArchiveDir | Out-Null

Copy-Item -Path (Join-Path (Join-Path $repoRoot $CaptureDir) "*") -Destination $cameraArchiveDir -Recurse -Force

Write-Host "Archived camera debug snapshot for commit ${commit}:"
Write-Host " - camera debug: $cameraArchiveDir"
