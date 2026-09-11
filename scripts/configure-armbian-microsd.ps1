[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [Parameter(Mandatory = $true)]
    [string]$BusId,
    [string]$Distro = "Ubuntu",
    [string]$PartitionLabel = "armbi_root",
    [string]$MountPath = "/mnt/armbian-autoconfig",
    [string]$WifiSsid = "ThaiBuffet",
    [Parameter(Mandatory = $true)]
    [string]$WifiPassword,
    [string]$WifiCountryCode = "IL",
    [string]$UserName = "pi",
    [Parameter(Mandatory = $true)]
    [string]$UserPassword,
    [string]$RootPassword,
    [string]$RealName,
    [string]$Locale = "en_US.UTF-8",
    [string]$Timezone = "Asia/Jerusalem",
    [string]$UserShell = "bash",
    [switch]$SkipUsbAttach,
    [switch]$KeepAttached
)

$ErrorActionPreference = "Stop"

function Test-IsAdministrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Invoke-Native {
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

function Get-UsbipdDeviceState {
    param(
        [Parameter(Mandatory = $true)]
        [string]$TargetBusId
    )

    $lines = & usbipd.exe list
    if ($LASTEXITCODE -ne 0) {
        throw "usbipd list failed with exit code $LASTEXITCODE"
    }

    $escapedBusId = [regex]::Escape($TargetBusId)
    $line = $lines | Where-Object { $_ -match "^\s*$escapedBusId\s+" } | Select-Object -First 1
    if (-not $line) {
        throw "USB bus id '$TargetBusId' was not found. Run 'usbipd list' in an elevated PowerShell and pass the SD reader bus id."
    }

    if ($line -notmatch "\s(?<state>Not shared|Shared|Attached)$") {
        throw "Could not parse usbipd device state from: $line"
    }

    return $Matches.state
}

function ConvertTo-BashAssignment {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [AllowEmptyString()]
        [string]$Value
    )

    $escapedValue = $Value -replace "'", "'\''"
    return "$Name='$escapedValue'"
}

if (-not $RootPassword) {
    $RootPassword = $UserPassword
}
if (-not $RealName) {
    $RealName = $UserName
}

if (-not (Get-Command usbipd.exe -ErrorAction SilentlyContinue)) {
    throw "Missing usbipd.exe. Install it with: winget install --interactive --exact dorssel.usbipd-win"
}
if (-not (Get-Command wsl.exe -ErrorAction SilentlyContinue)) {
    throw "Missing wsl.exe. Install or enable WSL first."
}
if (-not $SkipUsbAttach -and -not (Test-IsAdministrator)) {
    throw "Run this script from an elevated PowerShell, or pass -SkipUsbAttach after attaching the device yourself."
}

$configLines = @(
    ConvertTo-BashAssignment -Name "PRESET_NET_CHANGE_DEFAULTS" -Value "1"
    ConvertTo-BashAssignment -Name "PRESET_NET_WIFI_ENABLED" -Value "1"
    ConvertTo-BashAssignment -Name "PRESET_NET_WIFI_SSID" -Value $WifiSsid
    ConvertTo-BashAssignment -Name "PRESET_NET_WIFI_KEY" -Value $WifiPassword
    ConvertTo-BashAssignment -Name "PRESET_NET_WIFI_COUNTRYCODE" -Value $WifiCountryCode
    ConvertTo-BashAssignment -Name "PRESET_CONNECT_WIRELESS" -Value "n"
    ConvertTo-BashAssignment -Name "PRESET_NET_USE_STATIC" -Value "0"
    ConvertTo-BashAssignment -Name "SET_LANG_BASED_ON_LOCATION" -Value "n"
    ConvertTo-BashAssignment -Name "PRESET_LOCALE" -Value $Locale
    ConvertTo-BashAssignment -Name "PRESET_TIMEZONE" -Value $Timezone
    ConvertTo-BashAssignment -Name "PRESET_ROOT_PASSWORD" -Value $RootPassword
    ConvertTo-BashAssignment -Name "PRESET_USER_NAME" -Value $UserName
    ConvertTo-BashAssignment -Name "PRESET_USER_PASSWORD" -Value $UserPassword
    ConvertTo-BashAssignment -Name "PRESET_DEFAULT_REALNAME" -Value $RealName
    ConvertTo-BashAssignment -Name "PRESET_USER_SHELL" -Value $UserShell
)
$configText = ($configLines -join "`n") + "`n"
$configBase64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($configText))

$attachedHere = $false
if (-not $SkipUsbAttach) {
    $state = Get-UsbipdDeviceState -TargetBusId $BusId
    if ($state -eq "Not shared") {
        if ($PSCmdlet.ShouldProcess("USB device $BusId", "Share with usbipd")) {
            Invoke-Native -FilePath "usbipd.exe" -Arguments @("bind", "--busid", $BusId)
        }
        $state = Get-UsbipdDeviceState -TargetBusId $BusId
    }

    if ($state -ne "Attached") {
        if ($PSCmdlet.ShouldProcess("USB device $BusId", "Attach to WSL distro $Distro")) {
            Invoke-Native -FilePath "usbipd.exe" -Arguments @("attach", "--wsl", $Distro, "--busid", $BusId)
            $attachedHere = $true
        }
    }
}

$linuxScript = @'
set -euo pipefail

mount_path="$1"
partition_label="$2"
config_base64="$3"

partition="$(blkid -L "$partition_label" 2>/dev/null || true)"
if [ -z "$partition" ]; then
    partition="$(lsblk -rnpo NAME,FSTYPE | awk '$2 == "ext4" { print $1; exit }')"
fi
if [ -z "$partition" ]; then
    echo "Could not find an ext4 Armbian partition. Current block devices:" >&2
    lsblk -o NAME,PATH,SIZE,FSTYPE,LABEL,MOUNTPOINTS >&2
    exit 1
fi

mkdir -p "$mount_path"
if mountpoint -q "$mount_path"; then
    echo "$mount_path is already mounted; refusing to overwrite an existing mount." >&2
    exit 1
fi

mount "$partition" "$mount_path"
cleanup() {
    sync || true
    if mountpoint -q "$mount_path"; then
        umount "$mount_path"
    fi
}
trap cleanup EXIT

if [ ! -d "$mount_path/root" ] || [ ! -d "$mount_path/boot" ]; then
    echo "Mounted $partition, but it does not look like an Armbian root filesystem." >&2
    exit 1
fi

printf '%s' "$config_base64" | base64 -d > "$mount_path/root/.not_logged_in_yet"
chmod 664 "$mount_path/root/.not_logged_in_yet"

echo "Configured Armbian autoconfig on $partition ($mount_path/root/.not_logged_in_yet)."
'@
$linuxScriptBase64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($linuxScript))
$linuxCommand = "printf '%s' '$linuxScriptBase64' | base64 -d | bash -s -- '$MountPath' '$PartitionLabel' '$configBase64'"

$configured = $false
try {
    if ($PSCmdlet.ShouldProcess("Armbian partition labeled $PartitionLabel", "Write first-boot autoconfig")) {
        Invoke-Native -FilePath "wsl.exe" -Arguments @("-d", $Distro, "-u", "root", "--", "bash", "-lc", $linuxCommand)
        $configured = $true
    }
}
finally {
    if ($attachedHere -and -not $KeepAttached) {
        Invoke-Native -FilePath "usbipd.exe" -Arguments @("detach", "--busid", $BusId)
    }
}

if ($configured) {
    Write-Host "Armbian microSD initial configuration complete."
}
