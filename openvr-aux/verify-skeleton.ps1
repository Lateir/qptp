param([int]$Seconds = 70)
$ErrorActionPreference = 'Stop'
if ($Seconds -lt 1 -or $Seconds -gt 600) { throw 'Seconds must be between 1 and 600.' }
$checker = Join-Path $PSScriptRoot 'build/Release/skeleton_check.exe'
if (!(Test-Path -LiteralPath $checker)) { throw 'Build the Release CMake targets first.' }
if (!(Get-Process vrserver -ErrorAction SilentlyContinue)) { throw 'Connect Virtual Desktop and start SteamVR first.' }
$paths = Get-Content -LiteralPath (Join-Path $env:LOCALAPPDATA 'openvr/openvrpaths.vrpath') -Raw | ConvertFrom-Json
$runtime = $paths.runtime | Where-Object { Test-Path -LiteralPath (Join-Path $_ 'bin/win64/openvr_api.dll') } | Select-Object -First 1
if (!$runtime) { throw 'SteamVR runtime was not found.' }
$output = Join-Path $env:TEMP ('qptp-skeleton-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.csv')
Write-Host 'Keep Skeleton input enabled for the first 50 seconds, then switch it off and on.'
Write-Host 'Move index fingers toward/away from the triggers and sideways; move thumbs around both pads.'
Write-Host "CSV: $output"
& $checker (Join-Path $runtime 'bin/win64/openvr_api.dll') $Seconds $output
$result = $LASTEXITCODE
Write-Host "Result: $result. CSV: $output"
if ($paths.log.Count -gt 0) {
    $log = Join-Path $paths.log[0] 'vrserver.txt'
    if (Test-Path -LiteralPath $log) {
        Get-Content -LiteralPath $log | Select-String '\[qptp\]|Not loading driver qptp' | Select-Object -Last 35
    }
}
exit $result
