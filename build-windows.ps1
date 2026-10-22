# Builds photopea-offline.exe (see README.md for installing the tools).
#
#   .\build-windows.ps1             first build on a computer (downloads Rust packages, needs internet)
#   .\build-windows.ps1 -Offline    later builds, no internet needed
#
# If Windows refuses to run scripts:  powershell -ExecutionPolicy Bypass -File .\build-windows.ps1
param([switch]$Offline)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$app = Join-Path $root 'src-tauri'
$exe = Join-Path $app 'target\release\photopea-offline.exe'
$started = Get-Date

# started by double-click / "Run with PowerShell": keep the window open at the end
$fromExplorer = $false
try {
    $parent = (Get-CimInstance Win32_Process -Filter "ProcessId=$PID").ParentProcessId
    $fromExplorer = (Get-Process -Id $parent -ErrorAction SilentlyContinue).Name -eq 'explorer'
} catch {}

function Step($n, $text) { Write-Host ""; Write-Host "[$n/5] $text" -ForegroundColor Cyan }
function Ok($text) { Write-Host "      OK  $text" -ForegroundColor Green }
function Finish($code) {
    if ($fromExplorer) { Write-Host ""; Read-Host 'Press Enter to close' | Out-Null }
    exit $code
}
function Fail($text) {
    Write-Host ""
    Write-Host "BUILD FAILED: $text" -ForegroundColor Red
    Write-Host "See README.md (Building / Troubleshooting)." -ForegroundColor Red
    Finish 1
}

Write-Host "Building Photopea Offline" -ForegroundColor White

# 1. tools
Step 1 'Checking the build tools'
foreach ($tool in 'cargo', 'rustc') {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) { Fail "'$tool' was not found. Install Rust (README, Building on Windows, step 1d) and open a new PowerShell window." }
}
Ok (& rustc --version)
Ok (& cargo --version)
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vc = $null
if (Test-Path $vswhere) { $vc = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property displayName }
if (-not $vc) { Fail "the Microsoft C++ Build Tools were not found. Install them with 'Desktop development with C++' (README, Building on Windows, step 1c)." }
Ok "C++ build tools: $vc"

# 2. project files
Step 2 'Checking the project files'
foreach ($f in 'src-tauri\Cargo.toml', 'src-tauri\Cargo.lock', 'src-tauri\src\main.rs', 'www\index.html', 'www\code\pp\pp1792573386.js') {
    if (-not (Test-Path (Join-Path $root $f))) { Fail "missing $f. Copy the whole project folder again." }
}
Ok 'source files present'

# 3. the program must not be running (Windows locks a running exe)
Step 3 'Checking that Photopea is not running'
if (Get-Process photopea-offline -ErrorAction SilentlyContinue) { Fail 'Photopea is running. Close it and run this script again.' }
Ok 'not running'

# 4. compile
$cargoArgs = @('build', '--release', '--locked')
if ($Offline) { $cargoArgs += '--offline' }
Step 4 ("Compiling (cargo " + ($cargoArgs -join ' ') + ")")
if ($Offline) { Write-Host '      Offline: using Rust packages already downloaded on this computer.' }
else { Write-Host '      The first build downloads Rust packages and takes 5-15 minutes; later builds take 1-3 minutes.' }
# keep this computer's paths (user name, folders) out of the exe: rewrite them to neutral names
$cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
$cargoArgs += @('--config', "build.rustflags=['--remap-path-prefix=$(Join-Path $cargoHome 'registry')=cargo','--remap-path-prefix=$app=src-tauri']")
Push-Location $app
try {
    & cargo @cargoArgs
    $code = $LASTEXITCODE
} finally { Pop-Location }
if ($code -ne 0) { Fail "cargo exited with code $code (see the messages above)." }
if (-not (Test-Path $exe)) { Fail "the build finished but $exe was not created." }
Ok 'compiled'

# 5. result
Step 5 'Done'
$info = Get-Item $exe
$took = (Get-Date) - $started
Write-Host ""
Write-Host "Saved to:  $($info.FullName)" -ForegroundColor Green
Write-Host ("Size:      {0:N1} MB    Built: {1}    Took: {2:mm\:ss}" -f ($info.Length / 1MB), $info.LastWriteTime, $took)
Write-Host "Fonts:     the fonts installed on this computer"
Finish 0
