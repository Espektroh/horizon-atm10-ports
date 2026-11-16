# Instala Powah (port 0.1.0) no Horizon, com backup antes.
# Uso (PowerShell, nesta pasta):
#   powershell -ExecutionPolicy Bypass -File .\instalar.ps1              # acha a pasta do jogo sozinho
#   powershell -ExecutionPolicy Bypass -File .\instalar.ps1 -GameDir "C:\caminho\da\pasta\do\jogo"
#   powershell -ExecutionPolicy Bypass -File .\instalar.ps1 -Verificar   # só confere o pacote e a instalação
param(
    [string]$GameDir = "",
    [switch]$Verificar
)
$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path

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
    # A pasta do jogo (saves, options.txt, mods): HORIZON_HOME, senao %APPDATA%\horizon, que e a
    # instancia padrao "Horizon". Instancias criadas no launcher ficam em instances\<nome>.
    $home_ = if ($env:HORIZON_HOME) { $env:HORIZON_HOME } else { Join-Path $env:APPDATA "horizon" }
    $dirs = @($home_)
    $inst = Join-Path $home_ "instances"
    if (Test-Path $inst) {
        $dirs += @(Get-ChildItem $inst -Directory | ForEach-Object { $_.FullName })
    }
    if ($dirs.Count -eq 1) { return $dirs[0] }
    Write-Host "Instancias encontradas:"
    for ($i = 0; $i -lt $dirs.Count; $i++) {
        $label = if ($i -eq 0) { "  (Horizon, a padrao)" } else { "" }
        Write-Host "  [$i] $($dirs[$i])$label"
    }
    $pick = Read-Host "Numero da instancia onde instalar (Enter = 0)"
    if ($pick -eq "") { $pick = 0 }
    return $dirs[[int]$pick]
}

if (-not (Test-Package)) { throw "Pacote corrompido: baixe de novo." }
if ($GameDir -eq "") { $GameDir = Find-GameDir }
New-Item -ItemType Directory -Force $GameDir | Out-Null
if (-not (Test-Path (Join-Path $GameDir "options.txt"))) {
    Write-Host "Aviso: $GameDir ainda nao tem options.txt (o jogo nunca abriu nessa instancia). Instalando mesmo assim." -ForegroundColor Yellow
}
Write-Host "Pasta do jogo: $GameDir"

$contentFiles = @("powah.hmod")
$nativeFiles = @("powah_native-0.1.0.hnmod")
$mods = Join-Path $GameDir "horizon-mods"
$native = Join-Path $GameDir "horizon-native-mods"

if ($Verificar) {
    $ok = $true
    foreach ($f in $contentFiles) { $p = Test-Path (Join-Path $mods $f); Write-Host ("{0,-40} {1}" -f $f, $(if ($p) { "instalado" } else { "FALTA" })); $ok = $ok -and $p }
    foreach ($f in $nativeFiles) { $p = Test-Path (Join-Path $native $f); Write-Host ("{0,-40} {1}" -f $f, $(if ($p) { "instalado" } else { "FALTA" })); $ok = $ok -and $p }
    exit $(if ($ok) { 0 } else { 1 })
}

New-Item -ItemType Directory -Force $mods, $native | Out-Null
$bk = Join-Path $GameDir ("mod-backups\antes-powah-" + (Get-Date -Format "yyyyMMdd-HHmmss"))
New-Item -ItemType Directory -Force $bk | Out-Null
$list0 = Join-Path $GameDir "horizon-mods.txt"
if (Test-Path $list0) { Copy-Item $list0 $bk }
Write-Host "Backup de horizon-mods.txt em $bk"
foreach ($f in $contentFiles) { Copy-Item (Join-Path $here $f) $mods -Force }
foreach ($f in $nativeFiles) { Copy-Item (Join-Path $here $f) $native -Force }

# Liga os dois (conteudo e nativo) em horizon-mods.txt (o que Options > Mods grava).
$list = Join-Path $GameDir "horizon-mods.txt"
$lines = @()
if (Test-Path $list) { $lines = @(Get-Content $list) }
foreach ($id in @("powah", "powah_native")) {
    $lines = @($lines | Where-Object { $_ -notmatch "^mod\.$id\:" })
    $lines += "mod.${id}:true"
}
Set-Content $list $lines

Write-Host ""
Write-Host "Instalado." -ForegroundColor Green
Write-Host "Agora no launcher: Settings > Native mods (experimental) ligado, depois Build na pagina Mods."
Write-Host 'Clique Build (ou "Build anyway") e confie no codigo nativo se o launcher pedir. Depois abra o jogo e confira em Options > Mods.'
