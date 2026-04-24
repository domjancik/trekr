param(
    [Parameter(Mandatory = $true)]
    [string]$BaselineDir,
    [Parameter(Mandatory = $true)]
    [string]$CandidateDir,
    [string]$DiffOutputDir = "",
    [string[]]$Pages = @(
        "timeline",
        "timeline-focused",
        "mappings",
        "mappings-overlay",
        "midi-io",
        "routing"
    )
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

function Get-CanonicalBitmapBytes {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $source = [System.Drawing.Image]::FromFile($Path)
    try {
        $bitmap = New-Object System.Drawing.Bitmap(
            $source.Width,
            $source.Height,
            [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
        )
        try {
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            try {
                $graphics.DrawImage($source, 0, 0, $source.Width, $source.Height)
            }
            finally {
                $graphics.Dispose()
            }

            $rect = New-Object System.Drawing.Rectangle(0, 0, $bitmap.Width, $bitmap.Height)
            $data = $bitmap.LockBits(
                $rect,
                [System.Drawing.Imaging.ImageLockMode]::ReadOnly,
                [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
            )
            try {
                $byteCount = [Math]::Abs($data.Stride) * $bitmap.Height
                $bytes = New-Object byte[] $byteCount
                [Runtime.InteropServices.Marshal]::Copy($data.Scan0, $bytes, 0, $byteCount)
                return [pscustomobject]@{
                    Width  = $bitmap.Width
                    Height = $bitmap.Height
                    Stride = [Math]::Abs($data.Stride)
                    Bytes  = $bytes
                }
            }
            finally {
                $bitmap.UnlockBits($data)
            }
        }
        finally {
            $bitmap.Dispose()
        }
    }
    finally {
        $source.Dispose()
    }
}

function New-DiffBitmap {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Baseline,
        [Parameter(Mandatory = $true)]
        [object]$Candidate,
        [Parameter(Mandatory = $true)]
        [string]$OutputPath
    )

    $diffBitmap = New-Object System.Drawing.Bitmap(
        $Baseline.Width,
        $Baseline.Height,
        [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
    )

    for ($y = 0; $y -lt $Baseline.Height; $y++) {
        for ($x = 0; $x -lt $Baseline.Width; $x++) {
            $offset = ($y * $Baseline.Stride) + ($x * 4)
            $changed = $false
            for ($channel = 0; $channel -lt 4; $channel++) {
                if ($Baseline.Bytes[$offset + $channel] -ne $Candidate.Bytes[$offset + $channel]) {
                    $changed = $true
                    break
                }
            }
            if ($changed) {
                $diffBitmap.SetPixel($x, $y, [System.Drawing.Color]::FromArgb(255, 255, 0, 0))
            } else {
                $diffBitmap.SetPixel($x, $y, [System.Drawing.Color]::FromArgb(255, 0, 0, 0))
            }
        }
    }

    try {
        $diffBitmap.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $diffBitmap.Dispose()
    }
}

$baselineRoot = (Resolve-Path $BaselineDir).Path
$candidateRoot = (Resolve-Path $CandidateDir).Path

if ($DiffOutputDir -ne "") {
    $null = New-Item -ItemType Directory -Force -Path $DiffOutputDir
    $diffRoot = (Resolve-Path $DiffOutputDir).Path
} else {
    $diffRoot = $null
}

$totalDiffPixels = 0L
$failures = @()

foreach ($page in $Pages) {
    $baselinePath = Join-Path $baselineRoot "$page.png"
    $candidatePath = Join-Path $candidateRoot "$page.png"

    if (-not (Test-Path $baselinePath)) {
        throw "Baseline screenshot missing: $baselinePath"
    }
    if (-not (Test-Path $candidatePath)) {
        throw "Candidate screenshot missing: $candidatePath"
    }

    $baseline = Get-CanonicalBitmapBytes -Path $baselinePath
    $candidate = Get-CanonicalBitmapBytes -Path $candidatePath

    if ($baseline.Width -ne $candidate.Width -or $baseline.Height -ne $candidate.Height) {
        $failures += [pscustomobject]@{
            Page          = $page
            DiffPixels    = -1
            Width         = "$($baseline.Width) vs $($candidate.Width)"
            Height        = "$($baseline.Height) vs $($candidate.Height)"
            BaselinePath  = $baselinePath
            CandidatePath = $candidatePath
        }
        Write-Host "DIFF $page size mismatch baseline=$($baseline.Width)x$($baseline.Height) candidate=$($candidate.Width)x$($candidate.Height)"
        continue
    }

    $diffPixels = 0L
    for ($y = 0; $y -lt $baseline.Height; $y++) {
        for ($x = 0; $x -lt $baseline.Width; $x++) {
            $offset = ($y * $baseline.Stride) + ($x * 4)
            for ($channel = 0; $channel -lt 4; $channel++) {
                if ($baseline.Bytes[$offset + $channel] -ne $candidate.Bytes[$offset + $channel]) {
                    $diffPixels++
                    break
                }
            }
        }
    }

    $totalPixels = [int64]$baseline.Width * [int64]$baseline.Height
    if ($diffPixels -eq 0) {
        Write-Host "OK   $page exact match ($totalPixels pixels)"
        continue
    }

    $totalDiffPixels += $diffPixels
    $failures += [pscustomobject]@{
        Page          = $page
        DiffPixels    = $diffPixels
        Width         = $baseline.Width
        Height        = $baseline.Height
        BaselinePath  = $baselinePath
        CandidatePath = $candidatePath
    }
    Write-Host "DIFF $page changed_pixels=$diffPixels total_pixels=$totalPixels"

    if ($diffRoot) {
        $diffPath = Join-Path $diffRoot "$page-diff.png"
        New-DiffBitmap -Baseline $baseline -Candidate $candidate -OutputPath $diffPath
        Write-Host "     diff image: $diffPath"
    }
}

if ($failures.Count -gt 0) {
    Write-Host ""
    Write-Host "Screenshot regression failed."
    $failures | Format-Table -AutoSize | Out-Host
    Write-Host "Total changed pixels: $totalDiffPixels"
    exit 1
}

Write-Host ""
Write-Host "Screenshot regression passed with zero pixel differences."
