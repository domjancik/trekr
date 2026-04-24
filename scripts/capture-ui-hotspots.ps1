param(
    [string]$OutputDir = "artifacts/hotspots",
    [string]$StateMode = "demo",
    [string]$StateFile = "",
    [string]$Theme = "",
    [string]$Script = "",
    [string[]]$Hotspot = @("transport-left", "transport-right", "status-strip", "timeline-header-controls", "fx-row"),
    [string]$CapturePadding = "8",
    [int]$Zoom = 8,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$resolvedHotspots = @()
foreach ($item in $Hotspot) {
    foreach ($part in ($item -split ",")) {
        $trimmed = $part.Trim()
        if ($trimmed -ne "") {
            $resolvedHotspots += $trimmed
        }
    }
}
if ($resolvedHotspots.Count -eq 0) {
    throw "At least one hotspot preset is required"
}

function Resolve-HotspotDefinition {
    param([string]$Name)

    switch ($Name.ToLowerInvariant()) {
        "transport-left" {
            return [pscustomobject]@{ preset = "transport-left"; region = "transport-left"; label = "Transport left controls" }
        }
        "transport-right" {
            return [pscustomobject]@{ preset = "transport-right"; region = "transport-right"; label = "Transport right status panel" }
        }
        "status-strip" {
            return [pscustomobject]@{ preset = "status-strip"; region = "status-strip"; label = "Active track status strip" }
        }
        "timeline-header-controls" {
            return [pscustomobject]@{ preset = "timeline-header-controls"; region = "timeline-header-controls"; label = "Timeline header controls" }
        }
        "fx-row" {
            return [pscustomobject]@{ preset = "fx-row"; region = "fx-row"; label = "FX row hotspot" }
        }
        default {
            throw "Unknown hotspot preset: $Name"
        }
    }
}

$scriptRoot = $PSScriptRoot
$repoRoot = Split-Path -Parent $scriptRoot
$outputRoot = Join-Path $repoRoot $OutputDir
$wrapperPath = Join-Path $scriptRoot "capture-ui-screens.ps1"
$binaryPath = Join-Path $repoRoot "target\debug\trekr.exe"

if (-not $SkipBuild) {
    & cargo build | Out-Host
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }
}

if (-not (Test-Path $binaryPath)) {
    throw "Expected capture binary not found at $binaryPath"
}

New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null

$hotspotEntries = @()
foreach ($presetName in $resolvedHotspots) {
    $definition = Resolve-HotspotDefinition -Name $presetName
    $tempRelative = Join-Path $OutputDir "_tmp\$($definition.preset)"
    $tempRoot = Join-Path $repoRoot $tempRelative
    if (Test-Path $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }

    $captureArgs = @{
        OutputDir = $tempRelative
        StateMode = $StateMode
        CaptureRegion = $definition.region
        CapturePadding = $CapturePadding
        CaptureZoom = $Zoom
        SkipBuild = $true
    }
    if ($StateFile -ne "") { $captureArgs.StateFile = $StateFile }
    if ($Theme -ne "") { $captureArgs.Theme = $Theme }
    if ($Script -ne "") { $captureArgs.Script = $Script }

    & $wrapperPath @captureArgs

    $tempManifestPath = Join-Path $tempRoot "manifest.json"
    $tempManifest = Get-Content $tempManifestPath -Raw | ConvertFrom-Json
    $sourceEntry = @($tempManifest.files | Where-Object {
        -not ($_.PSObject.Properties.Name -contains "focused_track_view") -or -not $_.focused_track_view
    })[0]
    if ($null -eq $sourceEntry) {
        $sourceEntry = @($tempManifest.files)[0]
    }
    $sourceBasePath = Join-Path $tempRoot $sourceEntry.filename
    $finalBaseName = "{0}.png" -f $definition.preset
    $finalBasePath = Join-Path $outputRoot $finalBaseName
    Copy-Item -LiteralPath $sourceBasePath -Destination $finalBasePath -Force

    $entry = [ordered]@{
        preset = $definition.preset
        label = $definition.label
        region = $tempManifest.capture_region
        region_name = $tempManifest.capture_region_name
        path = $finalBasePath
        width = $sourceEntry.width
        height = $sourceEntry.height
    }

    if ($sourceEntry.PSObject.Properties.Name -contains "zoom_path") {
        $zoomFilename = "{0}@{1}x.png" -f $definition.preset, $Zoom
        $finalZoomPath = Join-Path $outputRoot $zoomFilename
        Copy-Item -LiteralPath $sourceEntry.zoom_path -Destination $finalZoomPath -Force
        $entry.zoom_scale = $Zoom
        $entry.zoom_path = $finalZoomPath
    }

    $hotspotEntries += [pscustomobject]$entry

    Remove-Item -LiteralPath $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
}

$manifest = [pscustomobject]@{
    generated_at = [DateTimeOffset]::Now.ToString("o")
    output_dir = $outputRoot
    state_mode = $StateMode
    theme = if ($Theme -ne "") { $Theme } else { $null }
    script = if ($Script -ne "") { $Script } else { $null }
    capture_padding = $CapturePadding
    zoom = $Zoom
    hotspots = $hotspotEntries
}

$manifestPath = Join-Path $outputRoot "hotspots-manifest.json"
$manifest | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 $manifestPath

Write-Host "Captured UI hotspots:"
foreach ($entry in $hotspotEntries) {
    Write-Host (" - {0}: {1}" -f $entry.preset, $entry.path)
    if ($entry.PSObject.Properties.Name -contains "zoom_path") {
        Write-Host ("   zoom: {0}" -f $entry.zoom_path)
    }
}
Write-Host "Hotspot manifest: $(Resolve-Path $manifestPath)"
