[CmdletBinding()]
param(
    [string]$ConfigPath = ".\scripts\rpi-deploy.local.psd1",
    [string]$PiHost = "",
    [ValidateSet("TcpListen", "UdpPush")]
    [string]$Transport = "TcpListen",
    [string]$ObsHost = "",
    [int]$ObsPort = 1234,
    [int]$FrameRate = 30,
    [string]$Bitrate = "3500k",
    [string]$Preset = "ultrafast",
    [string]$DrmDevice = "/dev/dri/card0",
    [string]$CrtcId = "",
    [string]$PlaneId = "",
    [switch]$InstallFfmpeg,
    [switch]$DeployOnly,
    [switch]$Stop,
    [switch]$PromptForSudoPassword
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

function Get-RepoRoot {
    return Split-Path -Parent $PSScriptRoot
}

function Get-DeployConfig {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if (-not (Test-Path $Path)) {
        throw "Missing deploy config: $Path. Copy scripts/rpi-deploy.example.psd1 to scripts/rpi-deploy.local.psd1 and edit it."
    }

    $config = Import-PowerShellDataFile -Path $Path
    foreach ($requiredKey in @("Host", "User", "Port", "RemoteDir")) {
        if (-not $config.ContainsKey($requiredKey) -or [string]::IsNullOrWhiteSpace([string]$config[$requiredKey])) {
            throw "Deploy config is missing required key '$requiredKey': $Path"
        }
    }

    return $config
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

function Get-OpenScpArguments {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Config
    )

    $args = @("-P", [string]$Config.Port)
    if (-not [string]::IsNullOrWhiteSpace([string]$Config.SshKeyPath)) {
        $args += @("-i", [string]$Config.SshKeyPath)
    }
    return $args
}

function Get-PlinkArguments {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Config
    )

    $args = @("-P", [string]$Config.Port)
    if (-not [string]::IsNullOrWhiteSpace([string]$Config.SshKeyPath)) {
        $args += @("-i", [string]$Config.SshKeyPath)
    }
    if (-not [string]::IsNullOrWhiteSpace([string]$Config.Password)) {
        $args += @("-pw", [string]$Config.Password)
    }
    return $args
}

function Escape-BashSingleQuoted {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Value
    )

    return "'" + ($Value -replace "'", "'`"`'`"`'") + "'"
}

function Invoke-NativeChecked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath,
        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$FilePath $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
    }
}

function Invoke-RemoteCommand {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Config,
        [Parameter(Mandatory = $true)]
        [string]$Command
    )

    $userAtHost = "$($Config.User)@$($Config.Host)"
    if ([string]::IsNullOrWhiteSpace([string]$Config.Password)) {
        $sshArgs = @()
        $sshArgs += Get-OpenSshArguments -Config $Config
        $sshArgs += $userAtHost
        $sshArgs += $Command
        Invoke-NativeChecked -FilePath "ssh.exe" -Arguments $sshArgs
        return
    }

    $plink = Get-Command plink.exe -ErrorAction SilentlyContinue
    if (-not $plink) {
        throw "Password-based Pi access requires plink.exe on PATH. Install PuTTY or leave Password blank and use key-based OpenSSH auth."
    }

    $plinkArgs = @()
    $plinkArgs += Get-PlinkArguments -Config $Config
    $plinkArgs += $userAtHost
    $plinkArgs += $Command
    Invoke-NativeChecked -FilePath $plink.Source -Arguments $plinkArgs
}

function Copy-RemoteFile {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Config,
        [Parameter(Mandatory = $true)]
        [string]$LocalPath,
        [Parameter(Mandatory = $true)]
        [string]$RemoteDir
    )

    $userAtHost = "$($Config.User)@$($Config.Host)"
    $scpTarget = "${userAtHost}:$RemoteDir"
    if ([string]::IsNullOrWhiteSpace([string]$Config.Password)) {
        $scpArgs = @()
        $scpArgs += Get-OpenScpArguments -Config $Config
        $scpArgs += $LocalPath
        $scpArgs += $scpTarget
        Invoke-NativeChecked -FilePath "scp.exe" -Arguments $scpArgs
        return
    }

    $pscp = Get-Command pscp.exe -ErrorAction SilentlyContinue
    if (-not $pscp) {
        throw "Password-based Pi access requires pscp.exe on PATH. Install PuTTY or leave Password blank and use key-based OpenSSH auth."
    }

    $pscpArgs = @()
    $pscpArgs += Get-PlinkArguments -Config $Config
    $pscpArgs += $LocalPath
    $pscpArgs += $scpTarget
    Invoke-NativeChecked -FilePath $pscp.Source -Arguments $pscpArgs
}

function Get-LocalAddressForRoute {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RemoteHost,
        [Parameter(Mandatory = $true)]
        [int]$RemotePort
    )

    $socket = [System.Net.Sockets.Socket]::new(
        [System.Net.Sockets.AddressFamily]::InterNetwork,
        [System.Net.Sockets.SocketType]::Dgram,
        [System.Net.Sockets.ProtocolType]::Udp
    )
    try {
        $socket.Connect($RemoteHost, $RemotePort)
        return $socket.LocalEndPoint.Address.ToString()
    } finally {
        $socket.Dispose()
    }
}

function Read-SudoPassword {
    $secure = Read-Host -Prompt "Pi sudo password" -AsSecureString
    $bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
    try {
        return [Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr)
    } finally {
        [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)
    }
}

function Get-SudoRemoteCommand {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Config,
        [Parameter(Mandatory = $true)]
        [string]$Command,
        [string]$SudoPassword = ""
    )

    if (-not [string]::IsNullOrWhiteSpace($SudoPassword)) {
        $quotedPassword = Escape-BashSingleQuoted -Value $SudoPassword
        return "printf '%s\n' $quotedPassword | sudo -S -p '' $Command"
    }

    if (-not [string]::IsNullOrWhiteSpace([string]$Config.Password)) {
        $quotedPassword = Escape-BashSingleQuoted -Value ([string]$Config.Password)
        return "printf '%s\n' $quotedPassword | sudo -S -p '' $Command"
    }

    return "sudo -n $Command"
}

$repoRoot = Get-RepoRoot
$config = Get-DeployConfig -Path $ConfigPath

if (-not [string]::IsNullOrWhiteSpace($PiHost)) {
    $config.Host = $PiHost
}

if ($Transport -eq "UdpPush" -and [string]::IsNullOrWhiteSpace($ObsHost)) {
    $ObsHost = Get-LocalAddressForRoute -RemoteHost ([string]$config.Host) -RemotePort $ObsPort
}

$sudoPassword = ""
if ($PromptForSudoPassword -and ($InstallFfmpeg -or $Stop -or -not $DeployOnly)) {
    $sudoPassword = Read-SudoPassword
}

$streamScriptPath = Join-Path $repoRoot "scripts\stream-rpi-kmsgrab.sh"
if (-not (Test-Path $streamScriptPath)) {
    throw "Missing stream script: $streamScriptPath"
}

$remoteDir = [string]$config.RemoteDir
$remoteDirQuoted = Escape-BashSingleQuoted -Value $remoteDir
$remoteScript = "$remoteDir/stream-rpi-kmsgrab.sh"
$remoteScriptQuoted = Escape-BashSingleQuoted -Value $remoteScript

Invoke-RemoteCommand -Config $config -Command "mkdir -p $remoteDirQuoted"
Copy-RemoteFile -Config $config -LocalPath $streamScriptPath -RemoteDir $remoteDir
Invoke-RemoteCommand -Config $config -Command "chmod +x $remoteScriptQuoted"

if ($Stop) {
    $stopCommand = Get-SudoRemoteCommand -Config $config -Command "pkill -x ffmpeg || true" -SudoPassword $sudoPassword
    Invoke-RemoteCommand -Config $config -Command $stopCommand
    Write-Host "Stopped remote ffmpeg capture on $($config.Host)."
    exit 0
}

if ($InstallFfmpeg) {
    $installScript = Escape-BashSingleQuoted -Value "apt-get update && apt-get install -y ffmpeg"
    $installCommand = Get-SudoRemoteCommand -Config $config -Command "sh -c $installScript" -SudoPassword $sudoPassword
    Invoke-RemoteCommand -Config $config -Command $installCommand
}

$transportArg = if ($Transport -eq "TcpListen") { "tcp-listen" } else { "udp-push" }

$remoteArgs = @(
    "--transport", $transportArg,
    "--obs-port", [string]$ObsPort,
    "--framerate", [string]$FrameRate,
    "--bitrate", $Bitrate,
    "--preset", $Preset,
    "--drm-device", $DrmDevice
)

if ($Transport -eq "UdpPush") {
    $remoteArgs += @("--obs-host", $ObsHost)
}

if (-not [string]::IsNullOrWhiteSpace($CrtcId)) {
    $remoteArgs += @("--crtc-id", $CrtcId)
}
if (-not [string]::IsNullOrWhiteSpace($PlaneId)) {
    $remoteArgs += @("--plane-id", $PlaneId)
}

$remoteArgsQuoted = ($remoteArgs | ForEach-Object { Escape-BashSingleQuoted -Value $_ }) -join " "
$streamCommand = "$remoteScriptQuoted $remoteArgsQuoted"

Write-Host "Pi: $($config.User)@$($config.Host)"
Write-Host "Transport: $Transport"
if ($Transport -eq "UdpPush") {
    Write-Host "OBS receiver: ${ObsHost}:${ObsPort}"
    Write-Host "OBS Media Source input: udp://@0.0.0.0:${ObsPort}"
} else {
    Write-Host "OBS Media Source input: tcp://$($config.Host):${ObsPort}"
}

if ($DeployOnly) {
    Write-Host "Deployed remote script: $remoteScript"
    Write-Host "Remote run command: sudo $streamCommand"
    exit 0
}

Write-Host "Starting remote stream. Keep this PowerShell window open; press Ctrl+C to stop."

$sudoStreamCommand = Get-SudoRemoteCommand -Config $config -Command $streamCommand -SudoPassword $sudoPassword
Invoke-RemoteCommand -Config $config -Command $sudoStreamCommand
