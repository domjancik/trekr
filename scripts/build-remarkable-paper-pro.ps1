param(
    [switch]$Release,
    [switch]$SdlUnixConsoleBuild,
    [string]$Target = "aarch64-unknown-linux-gnu",
    [string]$Binary = "trekr",
    [switch]$SkipRuntimeLibStaging
)

$ErrorActionPreference = "Stop"

function Convert-WindowsPathToWslPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $fullPath = [System.IO.Path]::GetFullPath($Path)
    if ($fullPath -notmatch '^(?<drive>[A-Za-z]):\\(?<rest>.*)$') {
        throw "Only local drive paths are supported for WSL path conversion: $fullPath"
    }

    $drive = $Matches.drive.ToLowerInvariant()
    $rest = $Matches.rest -replace '\\', '/'
    if ([string]::IsNullOrEmpty($rest)) {
        return "/mnt/$drive"
    }

    return "/mnt/$drive/$rest"
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$buildScriptPath = Join-Path $repoRoot "scripts\build-rpi-zero-2w.ps1"

if (-not (Test-Path $buildScriptPath)) {
    throw "Missing shared ARM64 build script: $buildScriptPath"
}

Write-Host "Building reMarkable-target artifact via shared ARM64 Linux cross-build path..."
& $buildScriptPath -Release:$Release -SdlUnixConsoleBuild:$SdlUnixConsoleBuild -Target $Target -Binary $Binary

if (-not $SkipRuntimeLibStaging -and $Target -eq "aarch64-unknown-linux-gnu") {
    $profile = if ($Release) { "release" } else { "debug" }
    $artifactDir = Join-Path $repoRoot "target\$Target\$profile"
    $artifactDirLinux = Convert-WindowsPathToWslPath -Path $artifactDir
$stageCommand = @(
    "set -euo pipefail",
    "if [ -f /usr/lib/aarch64-linux-gnu/libasound.so.2 ]; then",
    "  cp -L /usr/lib/aarch64-linux-gnu/libasound.so.2 '$artifactDirLinux/libasound.so.2'",
    "fi",
    "if [ -f /usr/lib/aarch64-linux-gnu/libgbm.so.1 ]; then",
    "  cp -L /usr/lib/aarch64-linux-gnu/libgbm.so.1 '$artifactDirLinux/libgbm.so.1'",
    "fi",
    "if [ -f /usr/lib/aarch64-linux-gnu/gbm/dri_gbm.so ]; then",
    "  mkdir -p '$artifactDirLinux/gbm'",
    "  cp -L /usr/lib/aarch64-linux-gnu/gbm/dri_gbm.so '$artifactDirLinux/gbm/dri_gbm.so'",
    "fi",
    "cp -L /usr/lib/aarch64-linux-gnu/libgallium-*.so '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libLLVM.so.* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libsensors.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libxcb.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libxcb-randr.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libxcb-sync.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libxcb-present.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libxcb-xfixes.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libxcb-dri3.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libxshmfence.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libX11-xcb.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libdrm_amdgpu.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libelf.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libzstd.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libexpat.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libz.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libedit.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libncursesw.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libtinfo.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libXau.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libXdmcp.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libbsd.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libmd.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libEGL.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libEGL_mesa.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libGLESv2.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libglapi.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libGL.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libOpenGL.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libGLdispatch.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libGLX.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libdrm.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /usr/lib/aarch64-linux-gnu/libudev.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /lib/aarch64-linux-gnu/libsensors.so* '$artifactDirLinux/' 2>/dev/null || true",
    "cp -L /lib/aarch64-linux-gnu/libudev.so* '$artifactDirLinux/' 2>/dev/null || true"
) -join "`n"
    & wsl.exe bash -lc $stageCommand | Out-Host
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to stage runtime libraries from WSL"
    }
    Write-Host "Staged runtime libraries (when available): $artifactDir\\libasound.so.2, $artifactDir\\libgbm.so.1, $artifactDir\\gbm\\dri_gbm.so, Mesa/LLVM/XCB dependency set"
}
