# Instala Torchmaster ( port para Horizon 66f3981) com backup antes.
# Uso (PowerShell, nesta pasta):
#   powershell -ExecutionPolicy Bypass -File .\instalar.ps1 -Apenas torchmaster   # so o Torchmaster
#   powershell -ExecutionPolicy Bypass -File .\instalar.ps1 -GameDir "C:\caminho\da\pasta\do\jogo"
#   powershell -ExecutionPolicy Bypass -File .\instalar.ps1 -Verificar            # so confere o pacote e a instalacao
param(
    [string]$GameDir = "",
    [ValidateSet("", "torchmaster")]
    [string]$Apenas = "",
    [switch]$Verificar
)
$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path

function Test-Package {
    $ok = $true
    foreach ($line in Get-Content (Join-Path $here "SHA256SUMS.txt")) {
        if ($line.Trim() -eq "") { continue }
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

# Cada mod: conteudo (.hmod), nativo (.hnmod) e os ids que vao em horizon-mods.txt.
$all = @(
    @{ Name = "torchmaster"; Content = "torchmaster.hmod"; Native = "torchmaster_native-0.1.0.hnmod"; Ids = @("torchmaster", "torchmaster_native") }
)
$chosen = @($all | Where-Object { $Apenas -eq "" -or $_.Name -eq $Apenas })

if (-not (Test-Package)) { throw "Pacote corrompido: baixe de novo." }
if ($GameDir -eq "") { $GameDir = Find-GameDir }
New-Item -ItemType Directory -Force $GameDir | Out-Null
if (-not (Test-Path (Join-Path $GameDir "options.txt"))) {
    Write-Host "Aviso: $GameDir ainda nao tem options.txt (o jogo nunca abriu nessa instancia). Instalando mesmo assim." -ForegroundColor Yellow
}
Write-Host "Pasta do jogo: $GameDir"

$mods = Join-Path $GameDir "horizon-mods"
$native = Join-Path $GameDir "horizon-native-mods"

if ($Verificar) {
    $ok = $true
    foreach ($m in $chosen) {
        $p = Test-Path (Join-Path $mods $m.Content); Write-Host ("{0,-36} {1}" -f $m.Content, $(if ($p) { "instalado" } else { "FALTA" })); $ok = $ok -and $p
        $p = Test-Path (Join-Path $native $m.Native); Write-Host ("{0,-36} {1}" -f $m.Native, $(if ($p) { "instalado" } else { "FALTA" })); $ok = $ok -and $p
    }
    exit $(if ($ok) { 0 } else { 1 })
}

New-Item -ItemType Directory -Force $mods, $native | Out-Null
$bk = Join-Path $GameDir ("mod-backups\antes-utilidades-atm-" + (Get-Date -Format "yyyyMMdd-HHmmss"))
New-Item -ItemType Directory -Force $bk | Out-Null
$list = Join-Path $GameDir "horizon-mods.txt"
if (Test-Path $list) { Copy-Item $list $bk }
Write-Host "Backup de horizon-mods.txt em $bk"
foreach ($m in $chosen) {
    Copy-Item (Join-Path $here $m.Content) $mods -Force
    Copy-Item (Join-Path $here $m.Native) $native -Force
}

# Liga os mods escolhidos em horizon-mods.txt (o que Options > Mods grava).
$lines = @()
if (Test-Path $list) { $lines = @(Get-Content $list) }
foreach ($m in $chosen) {
    foreach ($id in $m.Ids) {
        $lines = @($lines | Where-Object { $_ -notmatch "^mod\.$id\:" })
        $lines += "mod.${id}:true"
    }
}
Set-Content $list $lines

Write-Host ""
Write-Host "Instalado." -ForegroundColor Green
Write-Host "Agora no launcher: Settings > Native mods (experimental) ligado, depois Build na pagina Mods."
Write-Host "Clique Build (ou Build anyway) e confie no codigo nativo se o launcher pedir. Depois abra o jogo e confira em Options > Mods."
