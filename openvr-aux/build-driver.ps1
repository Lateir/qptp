$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$vcvars = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
if (!(Test-Path -LiteralPath $vcvars)) { throw 'Visual Studio 2022 C++ Build Tools are required.' }
$output = Join-Path $root 'driver_qptp\bin\win64'
$intermediate = Join-Path $root 'build-msvc'
New-Item -ItemType Directory -Force -Path $output,$intermediate | Out-Null
$command = 'call "' + $vcvars + '" >nul && cd /d "' + $intermediate + '" && cl /nologo /std:c++17 /EHsc /MD /O2 /LD /I"' + (Join-Path $root 'vendor\openvr') + '" "' + (Join-Path $root 'src\driver.cpp') + '" "' + (Join-Path $root 'src\debug_udp.cpp') + '" ws2_32.lib /link /OUT:"' + (Join-Path $output 'driver_qptp.dll') + '"'
cmd.exe /d /s /c $command
if ($LASTEXITCODE -ne 0) { throw "Driver build failed: $LASTEXITCODE" }
