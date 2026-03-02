param(
    [switch]$Release,
    [switch]$SdlUnixConsoleBuild,
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
$sdlUnixConsoleBuildValue = if ($SdlUnixConsoleBuild) { "ON" } else { "OFF" }

$linuxCommand = @(
    "set -euo pipefail"
    "cd '$linuxRepoRoot'"
    'if [ -f "$HOME/.cargo/env" ]; then . "$HOME/.cargo/env"; fi'
    'command -v cargo >/dev/null 2>&1 || { echo "Missing WSL prerequisite: cargo" >&2; exit 1; }'
    'command -v rustup >/dev/null 2>&1 || { echo "Missing WSL prerequisite: rustup" >&2; exit 1; }'
    'command -v aarch64-linux-gnu-gcc >/dev/null 2>&1 || { echo "Missing WSL prerequisite: aarch64-linux-gnu-gcc" >&2; exit 1; }'
    'command -v aarch64-linux-gnu-g++ >/dev/null 2>&1 || { echo "Missing WSL prerequisite: aarch64-linux-gnu-g++" >&2; exit 1; }'
    'command -v aarch64-linux-gnu-ar >/dev/null 2>&1 || { echo "Missing WSL prerequisite: aarch64-linux-gnu-ar" >&2; exit 1; }'
    'command -v cmake >/dev/null 2>&1 || { echo "Missing WSL prerequisite: cmake" >&2; exit 1; }'
    'command -v ninja >/dev/null 2>&1 || { echo "Missing WSL prerequisite: ninja" >&2; exit 1; }'
    'command -v pkg-config >/dev/null 2>&1 || { echo "Missing WSL prerequisite: pkg-config" >&2; exit 1; }'
    "if ! rustup target list --installed | grep -qx '$Target'; then echo 'Rust target $Target is not installed in WSL.' >&2; echo 'Run inside WSL: rustup target add $Target' >&2; exit 1; fi"
    'export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc'
    'export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++'
    'export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar'
    'export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-g++'
    'export PKG_CONFIG_ALLOW_CROSS=1'
    'export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig'
    'export PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig'
    "export SDL_UNIX_CONSOLE_BUILD=$sdlUnixConsoleBuildValue"
    "cargo $($cargoArgs -join ' ')"
) -join '; '
$encodedLinuxCommand = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($linuxCommand))

& wsl.exe bash -lc "printf '%s' '$encodedLinuxCommand' | base64 -d | bash" | Out-Host
if ($LASTEXITCODE -ne 0) {
    throw "WSL cross-build failed with exit code $LASTEXITCODE"
}

$artifactPath = Join-Path $repoRoot "target\$Target\$profile\$Binary"
Write-Host "Built Linux ARM64 artifact: $artifactPath"
