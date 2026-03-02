[CmdletBinding()]
param(
    [string]$ConfigPath = ".\scripts\pi-camera-debug.local.psd1",
    [string]$OutputDir = "artifacts/camera-debug",
    [string]$OutputName = "pi-output-clip.mkv",
    [string]$DeviceName = "",
    [string]$DeviceInput = "",
    [string]$VideoSize = "",
    [int]$FrameRate = 0,
    [string]$VideoCodec = "",
    [string]$PixelFormat = "",
    [int]$DurationSeconds = 8
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

function Get-RepoRoot {
    Split-Path -Parent $PSScriptRoot
}

function Get-OptionalConfig {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if (-not (Test-Path $Path)) {
        return @{}
    }

    Import-PowerShellDataFile -Path $Path
}

function Resolve-Setting {
    param(
        [Parameter(Mandatory = $true)]
        $Value,
        [Parameter(Mandatory = $true)]
        $Fallback
    )

    if ($null -eq $Value) {
        return $Fallback
    }

    if ($Value -is [string] -and [string]::IsNullOrWhiteSpace($Value)) {
        return $Fallback
    }

    if ($Value -is [int] -and $Value -le 0) {
        return $Fallback
    }

    return $Value
}

$repoRoot = Get-RepoRoot
$config = Get-OptionalConfig -Path $ConfigPath

$deviceName = Resolve-Setting -Value $DeviceName -Fallback (Resolve-Setting -Value $config.DeviceName -Fallback "usb video")
$deviceInput = Resolve-Setting -Value $DeviceInput -Fallback (Resolve-Setting -Value $config.DeviceInput -Fallback "")
$videoSize = Resolve-Setting -Value $VideoSize -Fallback (Resolve-Setting -Value $config.VideoSize -Fallback "1920x1080")
$frameRate = Resolve-Setting -Value $FrameRate -Fallback (Resolve-Setting -Value $config.FrameRate -Fallback 60)
$videoCodec = Resolve-Setting -Value $VideoCodec -Fallback (Resolve-Setting -Value $config.VideoCodec -Fallback "")
$pixelFormat = Resolve-Setting -Value $PixelFormat -Fallback (Resolve-Setting -Value $config.PixelFormat -Fallback "")
$durationSeconds = [Math]::Max(1, $DurationSeconds)
$videoInput = if ([string]::IsNullOrWhiteSpace($deviceInput)) {
    "video=$deviceName"
} elseif ($deviceInput.StartsWith("video=")) {
    $deviceInput
} else {
    "video=$deviceInput"
}

if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    throw "ffmpeg was not found on PATH. Install ffmpeg before using the camera capture flow."
}

$outputRoot = Join-Path $repoRoot $OutputDir
$outputPath = Join-Path $outputRoot $OutputName
New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
Remove-Item $outputPath -Force -ErrorAction SilentlyContinue

$ffmpegArgs = @(
    "-hide_banner",
    "-y",
    "-f", "dshow",
    "-video_size", $videoSize,
    "-framerate", [string]$frameRate
)

if (-not [string]::IsNullOrWhiteSpace($videoCodec)) {
    $ffmpegArgs += @("-vcodec", $videoCodec)
} elseif (-not [string]::IsNullOrWhiteSpace($pixelFormat)) {
    $ffmpegArgs += @("-pixel_format", $pixelFormat)
}

$ffmpegArgs += @(
    "-i", $videoInput,
    "-t", [string]$durationSeconds
)

if (-not [string]::IsNullOrWhiteSpace($videoCodec)) {
    $ffmpegArgs += @("-c:v", "copy")
} else {
    $ffmpegArgs += @("-c:v", "ffv1", "-level", "3")
}

$ffmpegArgs += $outputPath

& ffmpeg @ffmpegArgs
if ($LASTEXITCODE -ne 0 -or -not (Test-Path $outputPath)) {
    throw "Camera clip capture failed for '$deviceName' with exit code $LASTEXITCODE"
}

Write-Host "Captured Pi output clip:"
Write-Host " - clip: $outputPath"
