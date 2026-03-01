param(
    [switch]$Release,
    [string]$Target = "aarch64-unknown-linux-gnu",
    [string]$Binary = "trekr"
)

$ErrorActionPreference = "Stop"

function Convert-WindowsPathToWslPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $fullPath = [System.IO.Path]::GetFullPath($Path)
    if ($fullPath -notmatch '^(?<drive>[A-Za-z]):\\(?<rest>.*)$') {
        throw "Only local drive paths are supported for WSL builds: $fullPath"
    }

    $drive = $Matches.drive.ToLowerInvariant()
    $rest = $Matches.rest -replace '\\', '/'
    if ([string]::IsNullOrEmpty($rest)) {
        return "/mnt/$drive"
    }

    return "/mnt/$drive/$rest"
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$linuxRepoRoot = Convert-WindowsPathToWslPath -Path $repoRoot
$profile = if ($Release) { "release" } else { "debug" }
$cargoArgs = @("build", "--target", $Target)
if ($Release) {
    $cargoArgs += "--release"
}

$linuxCommand = @"
set -euo pipefail
cd '$linuxRepoRoot'

need_cmd() {
    if ! command -v "`$1" >/dev/null 2>&1; then
        echo "Missing WSL prerequisite: `$1" >&2
        exit 1
    fi
}

need_cmd cargo
need_cmd rustup
need_cmd aarch64-linux-gnu-gcc
need_cmd aarch64-linux-gnu-g++
need_cmd aarch64-linux-gnu-ar
need_cmd cmake
need_cmd ninja
need_cmd pkg-config

if ! rustup target list --installed | grep -qx '$Target'; then
    echo "Rust target $Target is not installed in WSL." >&2
    echo "Run inside WSL: rustup target add $Target" >&2
    exit 1
fi

export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-g++
export PKG_CONFIG_ALLOW_CROSS=1

cargo $($cargoArgs -join ' ')
"@

$linuxCommand | & wsl.exe bash -s -- | Out-Host
if ($LASTEXITCODE -ne 0) {
    throw "WSL cross-build failed with exit code $LASTEXITCODE"
}

$artifactPath = Join-Path $repoRoot "target\$Target\$profile\$Binary"
Write-Host "Built Linux ARM64 artifact: $artifactPath"
