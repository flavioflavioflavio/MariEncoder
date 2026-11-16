# tools/package_msix.ps1 — Pacote MSIX da Microsoft Store do Mari Encoder.
#
#   pwsh tools/package_msix.ps1 -IdentityName <...> -Publisher <...>   # pacote para a Store
#   pwsh tools/package_msix.ps1 -Register                              # teste local (Modo de Desenvolvedor)
#
# Identidade: Partner Center → Mari Encoder → Gerenciamento do produto →
# Identidade do produto. Até reservar o nome, -Register usa uma identidade de teste.
# A Store re-assina o pacote; nenhum certificado próprio é necessário.
# Saída: dist/MariEncoder-<versão>.msix (e dist/msix-stage com o layout).

param(
    [string]$IdentityName = "",
    # Mesmo publisher do MariPDF no Partner Center.
    [string]$Publisher = "CN=CDCC5894-95FC-42E2-9B2C-C3A22F788BE9",
    [string]$PublisherDisplayName = "Takemoto",
    # Versão do pacote (4 números, último = 0). Padrão: derivada do Cargo.toml.
    [string]$Version = "",
    [string]$Python = "python",
    [switch]$SkipBuild,
    # Registra o layout desempacotado para testar localmente (Modo de Desenvolvedor).
    [switch]$Register
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

if (-not $IdentityName) {
    if (-not $Register) { throw "Informe -IdentityName (Partner Center → Identidade do produto) ou use -Register para teste local." }
    $IdentityName = "Takemoto.MariEncoder.Dev"
}

$cargoVersion = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
if (-not $Version) {
    # X.Y.Z-rc.N → X.Y.(Z*100+N).0 ; X.Y.Z → X.Y.(Z*100+99).0 (RCs < estável, sempre crescente)
    if ($cargoVersion -notmatch '^(\d+)\.(\d+)\.(\d+)(?:-rc\.(\d+))?$') { throw "Versão não suportada: $cargoVersion" }
    $patch = [int]$Matches[3] * 100 + $(if ($Matches[4]) { [int]$Matches[4] } else { 99 })
    $Version = "$($Matches[1]).$($Matches[2]).$patch.0"
}
Write-Host "Mari Encoder $cargoVersion → pacote $Version ($IdentityName)"

$sdk = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.*\x64\makeappx.exe" |
    Sort-Object FullName | Select-Object -Last 1
if (-not $sdk) { throw "makeappx.exe não encontrado (instale o Windows SDK)" }

if (-not $SkipBuild) {
    cargo build --release --locked
    if ($LASTEXITCODE -ne 0) { throw "cargo build falhou" }
}
$exe = "target\release\MariEncoder.exe"
$dll = "target\release\MariEncoderMenu.dll"
foreach ($f in $exe, $dll) { if (-not (Test-Path $f)) { throw "Não encontrado: $f" } }

if (-not (Test-Path "vendor\ffmpeg\ffmpeg.exe")) { & pwsh -NoProfile -File tools\fetch_ffmpeg.ps1 }
if (-not (Test-Path "vendor\ffmpeg\ffmpeg.exe")) { throw "FFmpeg não encontrado em vendor\ffmpeg" }

$stage = "dist\msix-stage"
if (Test-Path $stage) {
    if ($Register) { Get-AppxPackage -Name $IdentityName | Remove-AppxPackage -ErrorAction SilentlyContinue }
    Remove-Item -Recurse -Force $stage
}
New-Item -ItemType Directory -Force "$stage\Assets", "$stage\ffmpeg" | Out-Null
Copy-Item $exe "$stage\MariEncoder.exe"
Copy-Item $dll "$stage\MariEncoderMenu.dll"
Copy-Item vendor\ffmpeg\* "$stage\ffmpeg\"
Copy-Item LICENSE "$stage\LICENSE.txt"
Copy-Item THIRD_PARTY_NOTICES.md "$stage\THIRD_PARTY_NOTICES.md"

& $Python tools/make_msix_assets.py "$stage\Assets"
if ($LASTEXITCODE -ne 0) { throw "geração dos ícones falhou" }

# Extensões com o menu de botão direito: as mesmas do config padrão do app.
$exts = (Get-Content shared\video_extensions.txt -Raw).Split([char[]]" `r`n`t", [StringSplitOptions]::RemoveEmptyEntries)
if ($exts.Count -lt 10) { throw "Lista de extensões vazia em shared\video_extensions.txt" }
$clsid = (Select-String -Path shared\brand.rs -Pattern 'CLSID_STORE: &str = "([0-9A-F-]+)"').Matches[0].Groups[1].Value
$itemTypes = ($exts | Sort-Object -Unique | ForEach-Object {
    "            <desktop5:ItemType Type=`".$_`"><desktop5:Verb Id=`"MariEncoder`" Clsid=`"$clsid`" /></desktop5:ItemType>"
}) -join "`r`n"

$manifest = Get-Content installer\msix\AppxManifest.xml.in -Raw -Encoding utf8
$manifest = $manifest.Replace("{{IDENTITY_NAME}}", $IdentityName).
    Replace("{{PUBLISHER}}", [Security.SecurityElement]::Escape($Publisher)).
    Replace("{{PUBLISHER_DISPLAY_NAME}}", [Security.SecurityElement]::Escape($PublisherDisplayName)).
    Replace("{{VERSION}}", $Version).
    Replace("{{CLSID}}", $clsid).
    Replace("{{ITEM_TYPES}}", $itemTypes)
[IO.File]::WriteAllText((Resolve-Path $stage).Path + "\AppxManifest.xml", $manifest, [Text.UTF8Encoding]::new($false))

# Índice de recursos (PRI): faz o Windows escolher o ícone certo para cada
# escala e tamanho (.scale-200, .targetsize-48 etc.).
$makepri = Join-Path $sdk.DirectoryName "makepri.exe"
$priConfig = Join-Path $env:TEMP "marienc-priconfig.xml"
& $makepri createconfig /cf $priConfig /dq pt-BR /pv 10.0.0 /o | Out-Null
if ($LASTEXITCODE -ne 0) { throw "makepri createconfig falhou" }
# Sem <packaging>: tudo num resources.pri só (o MSIX não é dividido em pacotes de recursos).
$config = Get-Content $priConfig -Raw
[IO.File]::WriteAllText($priConfig, ($config -replace '(?s)\s*<packaging>.*?</packaging>', ''))
& $makepri new /pr $stage /cf $priConfig /mn "$stage\AppxManifest.xml" /of "$stage\resources.pri" /o | Out-Null
if ($LASTEXITCODE -ne 0) { throw "makepri new falhou" }

if ($Register) {
    Add-AppxPackage -Register "$stage\AppxManifest.xml"
    Write-Host "Registrado: procure 'Mari Encoder' no menu Iniciar."
    return
}

$msix = "dist\MariEncoder-$cargoVersion.msix"
& $sdk.FullName pack /o /d $stage /p $msix | Out-Null
if ($LASTEXITCODE -ne 0) { throw "makeappx pack falhou" }
Write-Host ("{0} ({1:N1} MB)" -f $msix, ((Get-Item $msix).Length / 1MB))
