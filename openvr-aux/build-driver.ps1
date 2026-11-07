$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (!(Test-Path -LiteralPath $vswhere)) { throw 'Visual Studio Installer (vswhere.exe) is required.' }
$installation = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath | Select-Object -First 1
if (!$installation) { throw 'Visual Studio 2022 C++ tools are required.' }
$vcvars = Join-Path $installation 'VC\Auxiliary\Build\vcvars64.bat'
if (!(Test-Path -LiteralPath $vcvars)) { throw "vcvars64.bat is missing from $installation" }
$output = Join-Path $root 'driver_qptp\bin\win64'
$intermediate = Join-Path $root 'build-msvc'
New-Item -ItemType Directory -Force -Path $output,$intermediate | Out-Null
$sources = @('src/driver.cpp', 'src/debug_udp.cpp', 'src/fusion.cpp', 'src/skeletal_driver.cpp', 'src/skeletal_mapping.cpp', 'vendor/hand_simulation/hand_simulation.cpp') | ForEach-Object { '"' + (Join-Path $root $_) + '"' }
$cSources = @('vendor/minhook/src/buffer.c', 'vendor/minhook/src/hook.c', 'vendor/minhook/src/trampoline.c', 'vendor/minhook/src/hde/hde64.c') | ForEach-Object { '"' + (Join-Path $root $_) + '"' }
$command = 'call "' + $vcvars + '" >nul && cd /d "' + $intermediate + '" && cl /nologo /TC /MD /O2 /c /DWIN32_LEAN_AND_MEAN /DNOMINMAX ' + ($cSources -join ' ') + ' && cl /nologo /std:c++17 /EHsc /MD /O2 /LD /DWIN32_LEAN_AND_MEAN /DNOMINMAX /I"' + (Join-Path $root 'vendor/openvr') + '" /I"' + (Join-Path $root 'vendor/hand_simulation') + '" /I"' + (Join-Path $root 'vendor/minhook/include') + '" ' + ($sources -join ' ') + ' buffer.obj hook.obj trampoline.obj hde64.obj ws2_32.lib /link /OUT:"' + (Join-Path $output 'driver_qptp.dll') + '"'

cmd.exe /d /s /c $command
if ($LASTEXITCODE -ne 0) { throw "Driver build failed: $LASTEXITCODE" }
