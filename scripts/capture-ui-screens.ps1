param(
    [string]$OutputDir = "artifacts/screenshots",
    [string]$StateMode = "demo",
    [string]$StateFile = "",
    [string]$Script = "",
    [string]$Sequence = "",
    [string]$CaptureRegion = "",
    [string]$CaptureRect = "",
    [string]$CapturePadding = "",
    [string]$Annotate = ""
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
    "--capture-dir", $outputRoot,
    "--state-mode", $StateMode
)

if ($StateFile -ne "") {
    $statePath = if ([System.IO.Path]::IsPathRooted($StateFile)) {
        $StateFile
    } else {
        Join-Path $repoRoot $StateFile
    }
    $args += @("--state-file", $statePath)
}

if ($Script -ne "") {
    $scriptPath = if ([System.IO.Path]::IsPathRooted($Script)) {
        $Script
    } else {
        Join-Path $repoRoot $Script
    }
    $args += @("--script", $scriptPath)
}

if ($Sequence -ne "") {
    $sequencePath = if ([System.IO.Path]::IsPathRooted($Sequence)) {
        $Sequence
    } else {
        Join-Path $repoRoot $Sequence
    }
    $args += @("--sequence", $sequencePath)
}

if ($CaptureRegion -ne "") {
    $args += @("--capture-region", $CaptureRegion)
}

if ($CaptureRect -ne "") {
    $args += @("--capture-rect", $CaptureRect)
}

if ($CapturePadding -ne "") {
    $args += @("--capture-padding", $CapturePadding)
}

if ($Annotate -ne "") {
    $annotationPath = if ([System.IO.Path]::IsPathRooted($Annotate)) {
        $Annotate
    } else {
        Join-Path $repoRoot $Annotate
    }
    $args += @("--annotate", $annotationPath)
}

& $binaryPath @args | Out-Host
if ($LASTEXITCODE -ne 0) {
    throw "trekr UI capture failed with exit code $LASTEXITCODE"
}

$captures = Get-ChildItem -Path $outputRoot -Filter *.png | Sort-Object Name
$captureCount = @($captures).Count
if ($captureCount -eq 0) {
    throw "trekr UI capture produced no screenshots in $outputRoot"
}

$manifestPath = Join-Path $outputRoot "manifest.json"
$manifest = $null
if (Test-Path $manifestPath) {
    $manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
}

if ($null -eq $manifest) {
    $manifest = [pscustomobject]@{
        generated_at = [DateTimeOffset]::Now.ToString("o")
        output_dir = $outputRoot
        state_mode = $StateMode
        files = @(
            foreach ($capture in $captures) {
                [pscustomobject]@{
                    filename = $capture.Name
                    path = $capture.FullName
                    page = [System.IO.Path]::GetFileNameWithoutExtension($capture.Name)
                    width = 0
                    height = 0
                }
            }
        )
    }
    $manifest | ConvertTo-Json -Depth 6 | Set-Content -Encoding UTF8 $manifestPath
}

Write-Host "Captured renderer-level screenshots:"
foreach ($entry in $manifest.files) {
    $label = if ($entry.page) { $entry.page } else { [System.IO.Path]::GetFileNameWithoutExtension($entry.filename) }
    Write-Host (" - {0}: {1}" -f $label, $entry.path)
}
Write-Host "Manifest: $(Resolve-Path $manifestPath)"
