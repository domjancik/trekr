[CmdletBinding()]
param(
    [string]$ClipPath = ".\artifacts\camera-debug\remote-shutdown-flash.mkv",
    [string]$OutputDir = ".\artifacts\camera-debug\clip-analysis",
    [int]$Width = 160,
    [int]$Height = 90,
    [int]$Fps = 60
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    throw "ffmpeg was not found on PATH."
}

if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
    throw "python was not found on PATH."
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$resolvedClip = Resolve-Path -Path (Join-Path $repoRoot $ClipPath)
$outputRoot = Join-Path $repoRoot $OutputDir
$rawPath = Join-Path $outputRoot "frames.rgb"
$csvPath = Join-Path $outputRoot "metrics.csv"
$summaryPath = Join-Path $outputRoot "summary.txt"

New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
Remove-Item $rawPath,$csvPath,$summaryPath -Force -ErrorAction SilentlyContinue

$videoFilter = "fps=$Fps,scale=${Width}:${Height}"
& ffmpeg -hide_banner -y -i $resolvedClip -vf $videoFilter -pix_fmt rgb24 -f rawvideo $rawPath
if ($LASTEXITCODE -ne 0 -or -not (Test-Path $rawPath)) {
    throw "ffmpeg failed to extract raw frames from $resolvedClip"
}

$pythonScript = @'
import csv
import math
import os
import sys
from pathlib import Path

raw_path = Path(sys.argv[1])
csv_path = Path(sys.argv[2])
summary_path = Path(sys.argv[3])
width = int(sys.argv[4])
height = int(sys.argv[5])
fps = float(sys.argv[6])
frame_size = width * height * 3
data = raw_path.read_bytes()
if len(data) < frame_size:
    raise SystemExit("No frame data found")
frame_count = len(data) // frame_size
rows = []
prev = None
for index in range(frame_count):
    start = index * frame_size
    frame = data[start:start + frame_size]
    brightness_total = 0
    nonblack = 0
    diff_total = 0
    for offset in range(0, len(frame), 3):
        r = frame[offset]
        g = frame[offset + 1]
        b = frame[offset + 2]
        brightness_total += (r + g + b) / 3.0
        if (r + g + b) > 24:
            nonblack += 1
        if prev is not None:
            diff_total += abs(r - prev[offset]) + abs(g - prev[offset + 1]) + abs(b - prev[offset + 2])
    mean_brightness = brightness_total / (width * height)
    nonblack_ratio = nonblack / (width * height)
    diff = 0.0 if prev is None else diff_total / (width * height * 3.0)
    rows.append({
        "frame": index + 1,
        "time_s": index / fps,
        "mean_brightness": mean_brightness,
        "nonblack_ratio": nonblack_ratio,
        "frame_diff": diff,
    })
    prev = frame

with csv_path.open("w", newline="", encoding="utf-8") as handle:
    writer = csv.DictWriter(handle, fieldnames=["frame", "time_s", "mean_brightness", "nonblack_ratio", "frame_diff"])
    writer.writeheader()
    writer.writerows(rows)

top_changes = sorted(rows, key=lambda row: row["frame_diff"], reverse=True)[:10]
first_visible = next((row for row in rows if row["mean_brightness"] > 2.0 or row["nonblack_ratio"] > 0.01), None)
brightest = max(rows, key=lambda row: row["mean_brightness"])

with summary_path.open("w", encoding="utf-8") as handle:
    handle.write(f"frames={len(rows)}\n")
    handle.write(f"fps={fps}\n")
    if first_visible:
        handle.write(
            "first_visible="
            f"frame {first_visible['frame']} at {first_visible['time_s']:.3f}s "
            f"(mean={first_visible['mean_brightness']:.2f}, nonblack={first_visible['nonblack_ratio']:.4f})\n"
        )
    handle.write(
        "brightest="
        f"frame {brightest['frame']} at {brightest['time_s']:.3f}s "
        f"(mean={brightest['mean_brightness']:.2f}, nonblack={brightest['nonblack_ratio']:.4f})\n"
    )
    handle.write("top_changes=\n")
    for row in top_changes:
        handle.write(
            f"  frame {row['frame']} at {row['time_s']:.3f}s "
            f"diff={row['frame_diff']:.2f} mean={row['mean_brightness']:.2f} "
            f"nonblack={row['nonblack_ratio']:.4f}\n"
        )

print(summary_path.read_text(encoding="utf-8"), end="")
'@

@"
$pythonScript
"@ | python - $rawPath $csvPath $summaryPath $Width $Height $Fps

Write-Host "Clip analysis written to:"
Write-Host " - summary: $summaryPath"
Write-Host " - metrics: $csvPath"
