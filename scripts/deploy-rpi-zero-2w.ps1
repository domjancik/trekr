[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [string]$ConfigPath = ".\scripts\rpi-deploy.local.psd1",
    [switch]$SkipBuild,
    [switch]$BookwormBuild,
    [switch]$InstallRuntimeDeps,
    [switch]$StartAfterDeploy,
    [switch]$DeployMidiLoopbackHarness
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

    $targets = Get-SshTargets -Config $Config
    if ([string]::IsNullOrWhiteSpace([string]$Config.Password)) {
        $sshArgs = @()
        $sshArgs += Get-OpenSshArguments -Config $Config
        $sshArgs += $targets.UserAtHost
        $sshArgs += $Command
        Invoke-NativeChecked -FilePath "ssh.exe" -Arguments $sshArgs
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
    Invoke-NativeChecked -FilePath $plink.Source -Arguments $plinkArgs
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
        Invoke-NativeChecked -FilePath "scp.exe" -Arguments $scpArgs
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
    Invoke-NativeChecked -FilePath $pscp.Source -Arguments $pscpArgs
}

$repoRoot = Get-RepoRoot
$config = Get-DeployConfig -Path $ConfigPath
$targetRoot = if ($BookwormBuild) { "target\bookworm\aarch64-unknown-linux-gnu" } else { "target\aarch64-unknown-linux-gnu" }
$artifactPath = Join-Path $repoRoot "$targetRoot\release\trekr"
$loopbackHarnessPath = Join-Path $repoRoot "$targetRoot\release\trekr-midi-loopback-latency"
$sdlLibraryPath = Join-Path $repoRoot "$targetRoot\release\libSDL3.so.0"
$launchScriptPath = Join-Path $repoRoot "scripts\launch-rpi-zero-2w.sh"
$runtimeSetupPath = Join-Path $repoRoot "scripts\setup-rpi-zero-2w-runtime.sh"

if (-not $SkipBuild) {
    $buildScriptName = if ($BookwormBuild) { "build-rpi-zero-2w-bookworm.ps1" } else { "build-rpi-zero-2w.ps1" }
    $buildScriptPath = Join-Path $repoRoot "scripts\$buildScriptName"
    if ($PSCmdlet.ShouldProcess($artifactPath, "Build Pi Zero 2 W release artifact")) {
        & $buildScriptPath -Release -Binary trekr
    }
    if ($DeployMidiLoopbackHarness -and $PSCmdlet.ShouldProcess($loopbackHarnessPath, "Build Pi Zero 2 W MIDI loopback latency harness")) {
        & $buildScriptPath -Release -Binary trekr-midi-loopback-latency
    }
}

if (-not (Test-Path $artifactPath)) {
    throw "Missing build artifact: $artifactPath"
}
if (-not (Test-Path $sdlLibraryPath)) {
    throw "Missing SDL runtime library: $sdlLibraryPath"
}
if ($DeployMidiLoopbackHarness -and -not (Test-Path $loopbackHarnessPath)) {
    throw "Missing loopback harness artifact: $loopbackHarnessPath"
}
if (-not (Test-Path $runtimeSetupPath)) {
    throw "Missing runtime setup script: $runtimeSetupPath"
}

$remoteDirQuoted = Escape-BashSingleQuoted -Value ([string]$config.RemoteDir)
$remoteSetup = "mkdir -p $remoteDirQuoted && chmod 755 $remoteDirQuoted"
if ($PSCmdlet.ShouldProcess("$($config.User)@$($config.Host):$($config.RemoteDir)", "Prepare remote deployment directory")) {
    Invoke-RemoteCommand -Config $config -Command $remoteSetup
}

$pathsToCopy = @($artifactPath, $sdlLibraryPath, $launchScriptPath, $runtimeSetupPath)
if ($DeployMidiLoopbackHarness) {
    $pathsToCopy += $loopbackHarnessPath
}

if ($PSCmdlet.ShouldProcess("$($config.User)@$($config.Host):$($config.RemoteDir)", "Copy trekr binary, SDL runtime, and support scripts")) {
    Copy-RemoteFiles -Config $config -Paths $pathsToCopy
}

$remoteTrekr = Escape-BashSingleQuoted -Value "$($config.RemoteDir)/trekr"
$remoteLoopbackHarness = Escape-BashSingleQuoted -Value "$($config.RemoteDir)/trekr-midi-loopback-latency"
$remoteLauncher = Escape-BashSingleQuoted -Value "$($config.RemoteDir)/launch-rpi-zero-2w.sh"
$remoteRuntimeSetup = Escape-BashSingleQuoted -Value "$($config.RemoteDir)/setup-rpi-zero-2w-runtime.sh"
$remoteFinalizeTargets = @($remoteTrekr, $remoteLauncher, $remoteRuntimeSetup)
if ($DeployMidiLoopbackHarness) {
    $remoteFinalizeTargets += $remoteLoopbackHarness
}
$remoteFinalize = "chmod +x $($remoteFinalizeTargets -join ' ')"
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
