param(
    [string]$OutputDir = "artifacts/hotspots-ab",
    [string]$BaselineLabel = "baseline",
    [string]$CandidateLabel = "candidate",
    [string]$StateMode = "demo",
    [string]$StateFile = "",
    [string[]]$Hotspot = @("transport-left", "transport-right", "status-strip", "timeline-header-controls", "fx-row"),
    [string]$CapturePadding = "8",
    [int]$Zoom = 8,
    [string]$BaselineTheme = "",
    [string]$CandidateTheme = "",
    [string]$BaselineScript = "",
    [string]$CandidateScript = ""
)

$ErrorActionPreference = "Stop"

$scriptRoot = $PSScriptRoot
$repoRoot = Split-Path -Parent $scriptRoot
$outputRoot = Join-Path $repoRoot $OutputDir
$captureScript = Join-Path $scriptRoot "capture-ui-hotspots.ps1"

& cargo build | Out-Host
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
}

$baselineRelative = Join-Path $OutputDir $BaselineLabel
$candidateRelative = Join-Path $OutputDir $CandidateLabel

$sharedArgs = @{
    StateMode = $StateMode
    CapturePadding = $CapturePadding
    Zoom = $Zoom
    Hotspot = $Hotspot
    SkipBuild = $true
}

if ($StateFile -ne "") {
    $sharedArgs.StateFile = $StateFile
}

$baselineArgs = $sharedArgs.Clone()
$baselineArgs.OutputDir = $baselineRelative
if ($BaselineTheme -ne "") { $baselineArgs.Theme = $BaselineTheme }
if ($BaselineScript -ne "") { $baselineArgs.Script = $BaselineScript }

$candidateArgs = $sharedArgs.Clone()
$candidateArgs.OutputDir = $candidateRelative
if ($CandidateTheme -ne "") { $candidateArgs.Theme = $CandidateTheme }
if ($CandidateScript -ne "") { $candidateArgs.Script = $CandidateScript }

& $captureScript @baselineArgs
& $captureScript @candidateArgs

$baselineManifestPath = Join-Path (Join-Path $repoRoot $baselineRelative) "hotspots-manifest.json"
$candidateManifestPath = Join-Path (Join-Path $repoRoot $candidateRelative) "hotspots-manifest.json"
$baselineManifest = Get-Content $baselineManifestPath -Raw | ConvertFrom-Json
$candidateManifest = Get-Content $candidateManifestPath -Raw | ConvertFrom-Json

$pairs = @()
foreach ($baselineEntry in $baselineManifest.hotspots) {
    $candidateEntry = @($candidateManifest.hotspots | Where-Object { $_.preset -eq $baselineEntry.preset })[0]
    if ($null -eq $candidateEntry) {
        continue
    }
    $pairs += [pscustomobject]@{
        preset = $baselineEntry.preset
        label = $baselineEntry.label
        region = $baselineEntry.region
        region_name = $baselineEntry.region_name
        baseline_path = $baselineEntry.path
        candidate_path = $candidateEntry.path
        baseline_zoom_path = if ($baselineEntry.PSObject.Properties.Name -contains "zoom_path") { $baselineEntry.zoom_path } else { $null }
        candidate_zoom_path = if ($candidateEntry.PSObject.Properties.Name -contains "zoom_path") { $candidateEntry.zoom_path } else { $null }
    }
}

$comparisonManifest = [pscustomobject]@{
    generated_at = [DateTimeOffset]::Now.ToString("o")
    output_dir = $outputRoot
    baseline_label = $BaselineLabel
    candidate_label = $CandidateLabel
    state_mode = $StateMode
    capture_padding = $CapturePadding
    zoom = $Zoom
    baseline_manifest = $baselineManifestPath
    candidate_manifest = $candidateManifestPath
    pairs = $pairs
}

New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
$comparisonManifestPath = Join-Path $outputRoot "comparison-manifest.json"
$comparisonManifest | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 $comparisonManifestPath

Write-Host "Captured paired baseline/candidate hotspots:"
foreach ($pair in $pairs) {
    Write-Host (" - {0}" -f $pair.preset)
    Write-Host ("   baseline:  {0}" -f $pair.baseline_path)
    Write-Host ("   candidate: {0}" -f $pair.candidate_path)
}
Write-Host "Comparison manifest: $(Resolve-Path $comparisonManifestPath)"
