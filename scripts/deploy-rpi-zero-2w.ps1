[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [string]$ConfigPath = ".\scripts\rpi-deploy.local.psd1",
    [switch]$SkipBuild,
    [switch]$InstallRuntimeDeps,
    [switch]$StartAfterDeploy
)

$ErrorActionPreference = "Stop"

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

function Get-RepoRoot {
    return Split-Path -Parent $PSScriptRoot
}

function Get-SshTargets {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Config
    )

    $userAtHost = "$($Config.User)@$($Config.Host)"
    return @{
        UserAtHost = $userAtHost
        ScpTarget = "${userAtHost}:$($Config.RemoteDir)"
    }
}

function Get-OpenSshArguments {
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

function Invoke-RemoteCommand {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Config,
        [Parameter(Mandatory = $true)]
        [string]$Command
    )

    $targets = Get-SshTargets -Config $Config
    if ([string]::IsNullOrWhiteSpace([string]$Config.Password)) {
        $sshArgs = @()
        $sshArgs += Get-OpenSshArguments -Config $Config
        $sshArgs += $targets.UserAtHost
        $sshArgs += $Command
        & ssh.exe @sshArgs
        return
    }

    $plink = Get-Command plink.exe -ErrorAction SilentlyContinue
    if (-not $plink) {
        throw "Password-based deploy requires plink.exe on PATH. Install PuTTY or leave Password blank and use key-based OpenSSH auth."
    }

    $plinkArgs = @()
    $plinkArgs += Get-PlinkArguments -Config $Config
    $plinkArgs += $targets.UserAtHost
    $plinkArgs += $Command
    & $plink.Source @plinkArgs
}

function Copy-RemoteFiles {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Config,
        [Parameter(Mandatory = $true)]
        [string[]]$Paths
    )

    $targets = Get-SshTargets -Config $Config
    if ([string]::IsNullOrWhiteSpace([string]$Config.Password)) {
        $scpArgs = @()
        $scpArgs += Get-OpenSshArguments -Config $Config
        $scpArgs += $Paths
        $scpArgs += $targets.ScpTarget
        & scp.exe @scpArgs
        return
    }

    $pscp = Get-Command pscp.exe -ErrorAction SilentlyContinue
    if (-not $pscp) {
        throw "Password-based deploy requires pscp.exe on PATH. Install PuTTY or leave Password blank and use key-based OpenSSH auth."
    }

    $pscpArgs = @()
    $pscpArgs += Get-PlinkArguments -Config $Config
    $pscpArgs += $Paths
    $pscpArgs += $targets.ScpTarget
    & $pscp.Source @pscpArgs
}

$repoRoot = Get-RepoRoot
$config = Get-DeployConfig -Path $ConfigPath
$artifactPath = Join-Path $repoRoot "target\aarch64-unknown-linux-gnu\release\trekr"
$sdlLibraryPath = Join-Path $repoRoot "target\aarch64-unknown-linux-gnu\release\libSDL3.so.0"
$launchScriptPath = Join-Path $repoRoot "scripts\launch-rpi-zero-2w.sh"
$runtimeSetupPath = Join-Path $repoRoot "scripts\setup-rpi-zero-2w-runtime.sh"

if (-not $SkipBuild) {
    $buildScriptPath = Join-Path $repoRoot "scripts\build-rpi-zero-2w.ps1"
    if ($PSCmdlet.ShouldProcess($artifactPath, "Build Pi Zero 2 W release artifact")) {
        & $buildScriptPath -Release -SdlUnixConsoleBuild
    }
}

if (-not (Test-Path $artifactPath)) {
    throw "Missing build artifact: $artifactPath"
}
if (-not (Test-Path $sdlLibraryPath)) {
    throw "Missing SDL runtime library: $sdlLibraryPath"
}
if (-not (Test-Path $runtimeSetupPath)) {
    throw "Missing runtime setup script: $runtimeSetupPath"
}

$remoteDirQuoted = Escape-BashSingleQuoted -Value ([string]$config.RemoteDir)
$remoteSetup = "mkdir -p $remoteDirQuoted && chmod 755 $remoteDirQuoted"
if ($PSCmdlet.ShouldProcess("$($config.User)@$($config.Host):$($config.RemoteDir)", "Prepare remote deployment directory")) {
    Invoke-RemoteCommand -Config $config -Command $remoteSetup
}

if ($PSCmdlet.ShouldProcess("$($config.User)@$($config.Host):$($config.RemoteDir)", "Copy trekr binary, SDL runtime, and support scripts")) {
    Copy-RemoteFiles -Config $config -Paths @($artifactPath, $sdlLibraryPath, $launchScriptPath, $runtimeSetupPath)
}

$remoteTrekr = Escape-BashSingleQuoted -Value "$($config.RemoteDir)/trekr"
$remoteLauncher = Escape-BashSingleQuoted -Value "$($config.RemoteDir)/launch-rpi-zero-2w.sh"
$remoteRuntimeSetup = Escape-BashSingleQuoted -Value "$($config.RemoteDir)/setup-rpi-zero-2w-runtime.sh"
$remoteFinalize = "chmod +x $remoteTrekr $remoteLauncher $remoteRuntimeSetup"
if ($PSCmdlet.ShouldProcess("$($config.User)@$($config.Host):$($config.RemoteDir)", "Finalize remote file permissions")) {
    Invoke-RemoteCommand -Config $config -Command $remoteFinalize
}

if ($InstallRuntimeDeps) {
    if ([string]::IsNullOrWhiteSpace([string]$Config.Password)) {
        $remoteInstall = "sudo -n $remoteRuntimeSetup"
    } else {
        $sudoPassword = Escape-BashSingleQuoted -Value ([string]$Config.Password)
        $remoteInstall = "printf '%s\n' $sudoPassword | sudo -S -p '' $remoteRuntimeSetup"
    }

    if ($PSCmdlet.ShouldProcess("$($config.User)@$($config.Host):$($config.RemoteDir)", "Install Pi runtime package dependencies")) {
        Invoke-RemoteCommand -Config $config -Command $remoteInstall
    }
}

if ($StartAfterDeploy) {
    $remoteStart = "cd $remoteDirQuoted && exec ./launch-rpi-zero-2w.sh"
    if ($PSCmdlet.ShouldProcess("$($config.User)@$($config.Host):$($config.RemoteDir)", "Start trekr on the Pi")) {
        Invoke-RemoteCommand -Config $config -Command $remoteStart
    }
}
