param(
    [string]$OutputDir = "artifacts/screenshots",
    [int]$StartupDelayMs = 1200,
    [int]$PageDelayMs = 350
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$outputRoot = Join-Path $repoRoot $OutputDir
$binaryPath = Join-Path $repoRoot "target\debug\trekr.exe"

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$signature = @"
using System;
using System.Runtime.InteropServices;

public static class TrekrWin32 {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
}
"@

Add-Type -TypeDefinition $signature | Out-Null

function Wait-ForMainWindow {
    param(
        [System.Diagnostics.Process]$Process,
        [int]$TimeoutMs = 10000
    )

    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    while ($stopwatch.ElapsedMilliseconds -lt $TimeoutMs) {
        $Process.Refresh()
        if ($Process.HasExited) {
            throw "trekr exited before a main window appeared"
        }

        if ($Process.MainWindowHandle -ne 0) {
            return $Process.MainWindowHandle
        }

        Start-Sleep -Milliseconds 100
    }

    throw "timed out waiting for trekr main window"
}

function Set-AppFocus {
    param([IntPtr]$WindowHandle)

    [TrekrWin32]::ShowWindow($WindowHandle, 9) | Out-Null
    [TrekrWin32]::SetForegroundWindow($WindowHandle) | Out-Null
    Start-Sleep -Milliseconds 150
}

function Save-WindowScreenshot {
    param(
        [IntPtr]$WindowHandle,
        [string]$Path
    )

    $rect = New-Object TrekrWin32+RECT
    if (-not [TrekrWin32]::GetWindowRect($WindowHandle, [ref]$rect)) {
        throw "failed to read trekr window bounds"
    }

    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top
    if ($width -le 0 -or $height -le 0) {
        throw "trekr window bounds are invalid"
    }

    $bitmap = New-Object System.Drawing.Bitmap($width, $height)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
        $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

function Send-KeyChord {
    param([string]$Keys)

    [System.Windows.Forms.SendKeys]::SendWait($Keys)
    Start-Sleep -Milliseconds $PageDelayMs
}

if (-not (Test-Path $binaryPath)) {
    & cargo build | Out-Host
}

Get-Process trekr -ErrorAction SilentlyContinue | Stop-Process -Force
New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null

$process = Start-Process -FilePath $binaryPath -WorkingDirectory $repoRoot -PassThru

try {
    Start-Sleep -Milliseconds $StartupDelayMs
    $windowHandle = Wait-ForMainWindow -Process $process
    Set-AppFocus -WindowHandle $windowHandle

    $captures = @(
        @{ Name = "timeline"; Keys = "{F1}" },
        @{ Name = "mappings"; Keys = "{F2}" },
        @{ Name = "midi-io"; Keys = "{F3}" },
        @{ Name = "routing"; Keys = "{F4}" }
    )

    $manifest = @()
    foreach ($capture in $captures) {
        Send-KeyChord -Keys $capture.Keys
        $path = Join-Path $outputRoot "$($capture.Name).png"
        Save-WindowScreenshot -WindowHandle $windowHandle -Path $path
        $manifest += [pscustomobject]@{
            page = $capture.Name
            path = (Resolve-Path $path).Path
        }
    }

    $manifestPath = Join-Path $outputRoot "manifest.json"
    $manifest | ConvertTo-Json | Set-Content -Encoding UTF8 $manifestPath
    Write-Host "Captured screenshots:"
    $manifest | ForEach-Object { Write-Host " - $($_.page): $($_.path)" }
    Write-Host "Manifest: $(Resolve-Path $manifestPath)"
}
finally {
    if (-not $process.HasExited) {
        $process.CloseMainWindow() | Out-Null
        Start-Sleep -Milliseconds 300
    }
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force
    }
}
