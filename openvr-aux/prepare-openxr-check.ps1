$ErrorActionPreference = 'Stop'
# Official Khronos headers, pinned to a reviewed SDK revision. Build-only dependency.
$revision = '24a1f2b92d212501a2b1b8d30ffd53abf55fcfa8'
$include = Join-Path $PSScriptRoot 'build/openxr-include/openxr'
New-Item -ItemType Directory -Force -Path $include | Out-Null
foreach ($file in @('openxr.h', 'openxr_platform.h', 'openxr_platform_defines.h')) {
    Invoke-WebRequest -Uri "https://raw.githubusercontent.com/KhronosGroup/OpenXR-SDK/$revision/include/openxr/$file" -OutFile (Join-Path $include $file)
}
Write-Host 'Headers prepared. Configure and build openvr-aux/build with CMake to compile openxr_check.'
