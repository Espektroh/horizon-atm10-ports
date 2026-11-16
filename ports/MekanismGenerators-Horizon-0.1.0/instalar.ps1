# Instala o Mekanism Generators 0.1.0 no Horizon (Windows), junto com o mekanism.hmod corrigido (10.7.19-horizon.7).
# Precisa do Mekanism 0.6.x ja instalado (mekanism_native-0.6.1.hnmod em horizon-native-mods).
# Uso (PowerShell, nesta pasta):
#   powershell -ExecutionPolicy Bypass -File .\instalar.ps1              # acha a pasta do jogo sozinho
#   powershell -ExecutionPolicy Bypass -File .\instalar.ps1 -GameDir "C:\caminho\da\pasta\do\jogo"
#   powershell -ExecutionPolicy Bypass -File .\instalar.ps1 -Verificar   # so confere o pacote e a instalacao
param(
    [string]$GameDir = "",
    [switch]$Verificar
)
$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$nativeFile = "mekanismgenerators_native-0.1.0.hnmod"
$contentFiles = @("mekanismgenerators.hmod", "mekanism.hmod")

function Test-Package {
    $ok = $true
    foreach ($line in Get-Content (Join-Path $here "SHA256SUMS.txt")) {
        $parts = $line -split "\s+", 2
        $file = Join-Path $here $parts[1]
        if (-not (Test-Path $file)) { Write-Host "FALTA  $($parts[1])" -ForegroundColor Red; $ok = $false; continue }
        $hash = (Get-FileHash $file -Algorithm SHA256).Hash.ToLower()
        if ($hash -ne $parts[0]) { Write-Host "ALTERADO  $($parts[1])" -ForegroundColor Red; $ok = $false }
    }
    if ($ok) { Write-Host "Pacote integro (SHA-256 conferido)." -ForegroundColor Green }
    return $ok
}

function Find-GameDir {
    $direct = Join-Path $env:APPDATA "horizon"
    if (Test-Path (Join-Path $direct "horizon-mods")) { return $direct }
    $root = Join-Path $env:APPDATA "horizon-launcher"
    if (-not (Test-Path $root)) { throw "Nao achei $direct nem $root. Passe -GameDir." }
    $found = Get-ChildItem $root -Recurse -Filter "options.txt" -File -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -notmatch "\\(native|cache|trash|mod-browser)\\" } |
        Sort-Object LastWriteTime -Descending
    if (-not $found) { throw "Nao achei a pasta do jogo. Passe -GameDir." }
    $dirs = @($found | ForEach-Object { $_.DirectoryName } | Select-Object -Unique)
    if ($dirs.Count -eq 1) { return $dirs[0] }
    Write-Host "Achei mais de uma pasta de jogo (a mais recente primeiro):"
    for ($i = 0; $i -lt $dirs.Count; $i++) { Write-Host "  [$i] $($dirs[$i])" }
    $pick = Read-Host "Numero da instancia onde instalar (Enter = 0)"
    if ($pick -eq "") { $pick = 0 }
    return $dirs[[int]$pick]
}

if (-not (Test-Package)) { throw "Pacote corrompido: baixe de novo." }
if ($GameDir -eq "") { $GameDir = Find-GameDir }
Write-Host "Pasta do jogo: $GameDir"

$mods = Join-Path $GameDir "horizon-mods"
$native = Join-Path $GameDir "horizon-native-mods"

$mekNative = @(Get-ChildItem $native -Filter "mekanism_native*.hnmod" -ErrorAction SilentlyContinue)
if ($mekNative.Count -eq 0) {
    Write-Host "ATENCAO: nao achei o mekanism_native (Mekanism 0.6.x) em horizon-native-mods. Instale o Mekanism-Horizon-0.6.1 antes." -ForegroundColor Yellow
}

if ($Verificar) {
    $all = $true
    foreach ($f in @(@($mods, "mekanismgenerators.hmod"), @($mods, "mekanism.hmod"), @($native, $nativeFile))) {
        $path = Join-Path $f[0] $f[1]
        $ok = (Test-Path $path) -and ((Get-FileHash $path -Algorithm SHA256).Hash -eq (Get-FileHash (Join-Path $here $f[1]) -Algorithm SHA256).Hash)
        Write-Host ("{0,-40} {1}" -f $f[1], $(if ($ok) { "instalado (desta versao)" } else { "FALTA ou outra versao" }))
        $all = $all -and $ok
    }
    $old = @(Get-ChildItem $native -Filter "mekanismgenerators_native*.hnmod" -ErrorAction SilentlyContinue | Where-Object { $_.Name -ne $nativeFile })
    if ($old.Count -gt 0) { Write-Host "ATENCAO: versao antiga ainda em horizon-native-mods: $($old.Name -join ', ')" -ForegroundColor Yellow; $all = $false }
    exit $(if ($all) { 0 } else { 1 })
}

New-Item -ItemType Directory -Force $mods, $native | Out-Null
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$backup = Join-Path $GameDir "mekanismgenerators-backup-$stamp"
New-Item -ItemType Directory -Force $backup | Out-Null

# Outra versao do nativo sai: duas copias da mesma id quebram o build.
foreach ($old in @(Get-ChildItem $native -Filter "mekanismgenerators_native*.hnmod" -ErrorAction SilentlyContinue)) {
    Move-Item $old.FullName (Join-Path $backup $old.Name) -Force
    Write-Host "Versao anterior guardada: $($old.Name) -> $backup"
}
# O mekanism.hmod antigo (10.7.19-horizon.6) vai para o backup: ele reclama o namespace mekanismgenerators
# e, ligado junto, faria o Horizon deixar o mekanismgenerators.hmod de fora.
foreach ($name in $contentFiles) {
    $old = Join-Path $mods $name
    if (Test-Path $old) { Copy-Item $old (Join-Path $backup $name) -Force }
}
$list = Join-Path $GameDir "horizon-mods.txt"
if (Test-Path $list) { Copy-Item $list (Join-Path $backup "horizon-mods.txt") -Force }

foreach ($name in $contentFiles) { Copy-Item (Join-Path $here $name) $mods -Force }
Copy-Item (Join-Path $here $nativeFile) $native -Force

# Liga em horizon-mods.txt (o que Options > Mods grava).
$lines = @()
if (Test-Path $list) { $lines = @(Get-Content $list) }
foreach ($id in @("mekanismgenerators", "mekanism", "mekanismgenerators_native")) {
    $lines = @($lines | Where-Object { $_ -notmatch "^mod\.$id\:" })
    $lines += "mod.${id}:true"
}
Set-Content $list $lines

Write-Host ""
Write-Host "Instalado. Backup do que havia antes: $backup" -ForegroundColor Green
Write-Host "Agora no launcher: Build na pagina Mods (Settings > Native mods ligado). So o jogo e recompilado (alguns minutos)."
Write-Host "Para desfazer: apague os tres arquivos novos e copie de volta os do backup (o mekanism.hmod antigo inclusive)."
