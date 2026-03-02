[CmdletBinding()]
param(
    [string]$ConfigPath = ".\scripts\pi-camera-debug.local.psd1",
    [string]$OutputDir = "artifacts/camera-debug",
    [string]$OutputName = "pi-output.png",
    [string]$DeviceName = "",
    [string]$DeviceInput = "",
    [string]$VideoSize = "",
    [int]$FrameRate = 0,
    [string]$PixelFormat = "",
    [int]$SelectFrame = -1,
    [switch]$ListDevices,
    [switch]$ListOptions,
    [switch]$SkipPiStatus
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

function Get-RepoRoot {
    return Split-Path -Parent $PSScriptRoot
}

function Get-OptionalConfig {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if (-not (Test-Path $Path)) {
        return @{}
    }

    return Import-PowerShellDataFile -Path $Path
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

function Get-RpiDeployConfig {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RepoRoot
    )

    $path = Join-Path $RepoRoot "scripts\rpi-deploy.local.psd1"
    if (-not (Test-Path $path)) {
        return $null
    }

    return Import-PowerShellDataFile -Path $path
}

function Get-OpenSshArguments {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Config
    )

    $args = @("-p", [string]$Config.Port)
    if (-not [string]::IsNullOrWhiteSpace([string]$Config.SshKeyPath)) {
        $args += @("-i", [string]$Config.SshKeyPath)
    }
    return $args
}

function Write-PiStatus {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RepoRoot,
        [Parameter(Mandatory = $true)]
        [string]$OutputPath
    )

    $config = Get-RpiDeployConfig -RepoRoot $RepoRoot
    if ($null -eq $config) {
        return $false
    }

    $userAtHost = "$($config.User)@$($config.Host)"
    $remoteDir = [string]$config.RemoteDir
    $remoteCommand = "bash -lc ""echo host=`$(hostname); echo now=`$(date -Is); echo; echo process:; pgrep -a trekr || true; echo; echo appdir:; ls -l '$remoteDir' 2>/dev/null || true"""
    $sshArgs = @()
    $sshArgs += Get-OpenSshArguments -Config $config
    $sshArgs += $userAtHost
    $sshArgs += $remoteCommand

    $statusOutput = & ssh.exe @sshArgs 2>&1 | Out-String
    $statusOutput | Set-Content -Encoding UTF8 $OutputPath
    return $true
}

$repoRoot = Get-RepoRoot
$config = Get-OptionalConfig -Path $ConfigPath

$deviceName = Resolve-Setting -Value $DeviceName -Fallback (Resolve-Setting -Value $config.DeviceName -Fallback "Cam Link 4K")
$deviceInput = Resolve-Setting -Value $DeviceInput -Fallback (Resolve-Setting -Value $config.DeviceInput -Fallback "")
$videoSize = Resolve-Setting -Value $VideoSize -Fallback (Resolve-Setting -Value $config.VideoSize -Fallback "1920x1080")
$frameRate = Resolve-Setting -Value $FrameRate -Fallback (Resolve-Setting -Value $config.FrameRate -Fallback 60)
$pixelFormat = Resolve-Setting -Value $PixelFormat -Fallback (Resolve-Setting -Value $config.PixelFormat -Fallback "nv12")
$selectFrame = if ($SelectFrame -ge 0) { $SelectFrame } else { [int](Resolve-Setting -Value $config.SelectFrame -Fallback 0) }

if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    throw "ffmpeg was not found on PATH. Install ffmpeg before using the camera capture flow."
}

if ($ListDevices) {
    & ffmpeg -hide_banner -list_devices true -f dshow -i dummy
    if ($LASTEXITCODE -ne 0) {
        throw "ffmpeg device listing failed with exit code $LASTEXITCODE"
    }
    exit 0
}

if ($ListOptions) {
    & ffmpeg -hide_banner -list_options true -f dshow -i "video=$deviceName"
    if ($LASTEXITCODE -ne 0) {
        throw "ffmpeg option listing failed for '$deviceName' with exit code $LASTEXITCODE"
    }
    exit 0
}

$outputRoot = Join-Path $repoRoot $OutputDir
$outputPath = Join-Path $outputRoot $OutputName
$statusPath = Join-Path $outputRoot "pi-status.txt"
$manifestPath = Join-Path $outputRoot "manifest.json"

New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
Remove-Item $outputPath -Force -ErrorAction SilentlyContinue
Remove-Item $statusPath -Force -ErrorAction SilentlyContinue
Remove-Item $manifestPath -Force -ErrorAction SilentlyContinue

$ffmpegArgs = @(
    "-hide_banner",
    "-y",
    "-f", "dshow",
    "-video_size", $videoSize,
    "-framerate", [string]$frameRate,
    "-pixel_format", $pixelFormat,
    "-i", $(if ([string]::IsNullOrWhiteSpace($deviceInput)) { "video=$deviceName" } else { $deviceInput })
)

if ($selectFrame -gt 0) {
    $ffmpegArgs += @("-vf", "select=gte(n\,$selectFrame)")
}

$ffmpegArgs += @(
    "-update", "1",
    "-frames:v", "1",
    $outputPath
)

$stdoutPath = Join-Path $outputRoot "ffmpeg.stdout.log"
$stderrPath = Join-Path $outputRoot "ffmpeg.stderr.log"
Remove-Item $stdoutPath -Force -ErrorAction SilentlyContinue
Remove-Item $stderrPath -Force -ErrorAction SilentlyContinue

$ffmpegProcessArgs = $ffmpegArgs | ForEach-Object {
    if ($_ -match '\s') {
        '"' + ($_ -replace '"', '\"') + '"'
    } else {
        $_
    }
}

$process = Start-Process -FilePath "ffmpeg" -ArgumentList $ffmpegProcessArgs -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
$ffmpegOutput = @()
if (Test-Path $stdoutPath) {
    $ffmpegOutput += Get-Content $stdoutPath
}
if (Test-Path $stderrPath) {
    $ffmpegOutput += Get-Content $stderrPath
}
$ffmpegOutput = ($ffmpegOutput -join [Environment]::NewLine)

if ($process.ExitCode -ne 0 -or -not (Test-Path $outputPath)) {
    $extra = ""
    if ($ffmpegOutput -match "Could not run graph") {
        $extra = " The camera is likely in use by another application."
    }
    throw "Camera capture failed for '$deviceName'.$extra`n$ffmpegOutput"
}

$hasPiStatus = $false
if (-not $SkipPiStatus) {
    try {
        $hasPiStatus = Write-PiStatus -RepoRoot $repoRoot -OutputPath $statusPath
    } catch {
        $_.Exception.Message | Set-Content -Encoding UTF8 $statusPath
        $hasPiStatus = $true
    }
}

$manifest = [pscustomobject]@{
    captured_at = [DateTimeOffset]::Now.ToString("o")
    image_path = (Resolve-Path $outputPath).Path
    device_name = $deviceName
    device_input = $deviceInput
    video_size = $videoSize
    frame_rate = $frameRate
    pixel_format = $pixelFormat
    select_frame = $selectFrame
    pi_status_path = if ($hasPiStatus) { $statusPath } else { "" }
}

$manifest | ConvertTo-Json | Set-Content -Encoding UTF8 $manifestPath

Write-Host "Captured Pi output frame:"
Write-Host " - image: $outputPath"
Write-Host " - manifest: $manifestPath"
if ($hasPiStatus) {
    Write-Host " - pi status: $statusPath"
}
