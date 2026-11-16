# tools/fetch_ffmpeg.ps1 — Baixa o FFmpeg LGPL que vai dentro do pacote.
#
#   pwsh tools/fetch_ffmpeg.ps1
#
# Build "lgpl-shared" do BtbN (FFmpeg 9.0, sem x264/x265 e sem partes GPL), numa
# release fixa e conferida por SHA-256. Saída: vendor/ffmpeg/ (ffmpeg.exe,
# ffprobe.exe, DLLs e LICENSE.txt). vendor/ não vai para o repositório.

$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

$Tag = "autobuild-2026-11-15-12-40"
$Asset = "ffmpeg-n9.0.2-57-gd838f9f9ae-win64-lgpl-shared-9.0.zip"
$Sha256 = "6305750BB1B4E3F80D17A6644FEC1CF57EEF12106C4F9192DDB05E14A6B522B0"
$Url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/$Tag/$Asset"

$download = "vendor\download"
$zip = Join-Path $download $Asset
New-Item -ItemType Directory -Force $download | Out-Null
if (-not (Test-Path $zip) -or (Get-FileHash $zip).Hash -ne $Sha256) {
    Write-Host "Baixando $Asset..."
    Invoke-WebRequest $Url -OutFile $zip
}
$hash = (Get-FileHash $zip).Hash
if ($hash -ne $Sha256) { throw "SHA-256 não confere: $hash (esperado $Sha256)" }

$extract = "vendor\ffmpeg-extract"
if (Test-Path $extract) { Remove-Item -Recurse -Force $extract }
Expand-Archive $zip $extract
$root = (Get-ChildItem $extract -Directory | Select-Object -First 1).FullName

$out = "vendor\ffmpeg"
if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Force $out | Out-Null
# ffplay não é usado pelo app.
Get-ChildItem "$root\bin" -File | Where-Object { $_.Name -ne "ffplay.exe" } | Copy-Item -Destination $out
Copy-Item "$root\LICENSE.txt" "$out\LICENSE.txt"
Remove-Item -Recurse -Force $extract
$size = (Get-ChildItem $out -File | Measure-Object Length -Sum).Sum / 1MB
Write-Host ("FFmpeg pronto em {0} ({1:N0} MB)" -f $out, $size)
