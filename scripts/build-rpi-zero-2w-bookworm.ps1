param(
    [switch]$Release,
    [switch]$DesktopSdlBuild,
    [string]$Target = "aarch64-unknown-linux-gnu",
    [string]$Binary = "trekr",
    [string]$ImageName = "trekr-rpi-bookworm-builder",
    [string]$CargoRegistryVolume = "trekr-rpi-bookworm-cargo-registry",
    [string]$CargoGitVolume = "trekr-rpi-bookworm-cargo-git"
)

$ErrorActionPreference = "Stop"

function Convert-WindowsPathToDockerPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $fullPath = [System.IO.Path]::GetFullPath($Path)
    return $fullPath -replace '\\', '/'
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

if (-not (Get-Command docker.exe -ErrorAction SilentlyContinue)) {
    throw "Missing docker.exe. Install/start Docker Desktop or use scripts/build-rpi-zero-2w.ps1 for the WSL build path."
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$dockerfilePath = Join-Path $PSScriptRoot "Dockerfile.rpi-bookworm"
$dockerRepoRoot = Convert-WindowsPathToDockerPath -Path $repoRoot
$profile = if ($Release) { "release" } else { "debug" }
$cargoArgs = @("build", "--target", $Target, "--target-dir", "target/bookworm")
if ($Release) {
    $cargoArgs += "--release"
}
if (-not $DesktopSdlBuild) {
    $cargoArgs += @("--features", "sdl3/build-from-source-unix-console")
}
$linuxCommand = @(
    "set -euo pipefail"
    "export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc"
    "export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++"
    "export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar"
    "export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-g++"
    "export PKG_CONFIG_ALLOW_CROSS=1"
    "export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig"
    "export PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig"
    "cargo $($cargoArgs -join ' ')"
) -join "; "

Invoke-Native -FilePath "docker.exe" -Arguments @(
    "build",
    "-f",
    $dockerfilePath,
    "-t",
    $ImageName,
    $repoRoot
)

Invoke-Native -FilePath "docker.exe" -Arguments @(
    "volume",
    "create",
    $CargoRegistryVolume
)

Invoke-Native -FilePath "docker.exe" -Arguments @(
    "volume",
    "create",
    $CargoGitVolume
)

Invoke-Native -FilePath "docker.exe" -Arguments @(
    "run",
    "--rm",
    "-v",
    "${dockerRepoRoot}:/work",
    "-v",
    "${CargoRegistryVolume}:/opt/cargo/registry",
    "-v",
    "${CargoGitVolume}:/opt/cargo/git",
    "-w",
    "/work",
    $ImageName,
    "bash",
    "-lc",
    $linuxCommand
)

$artifactPath = Join-Path $repoRoot "target\bookworm\$Target\$profile\$Binary"
Write-Host "Built Debian Bookworm-compatible Linux ARM64 artifact: $artifactPath"
