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
$command = 'call "' + $vcvars + '" >nul && cd /d "' + $intermediate + '" && cl /nologo /std:c++17 /EHsc /MD /O2 /LD /I"' + (Join-Path $root 'vendor\openvr') + '" "' + (Join-Path $root 'src\driver.cpp') + '" "' + (Join-Path $root 'src\debug_udp.cpp') + '" ws2_32.lib /link /OUT:"' + (Join-Path $output 'driver_qptp.dll') + '"'
cmd.exe /d /s /c $command
if ($LASTEXITCODE -ne 0) { throw "Driver build failed: $LASTEXITCODE" }
