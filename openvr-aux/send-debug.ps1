param(
    [ValidateSet('stock', 'touchpad', 'one-button', 'two-buttons')]
    [string]$Mode = 'touchpad',
    [ValidateSet('left', 'right')]
    [string]$Side = 'left',
    [ValidateRange(-1, 1)][float]$X = 0.4,
    [ValidateRange(-1, 1)][float]$Y = -0.2,
    [ValidateRange(0, 1)][float]$Force = 0.8,
    [switch]$Touch,
    [switch]$Button1,
    [switch]$Button2
)
$lines = @(
    'left_pad_x=0', 'left_pad_y=0', 'left_pad_touch=0', 'left_pad_force=0',
    'right_pad_x=0', 'right_pad_y=0', 'right_pad_touch=0', 'right_pad_force=0',
    'left_single_button=0', 'left_double_button_1=0', 'left_double_button_2=0',
    'right_single_button=0', 'right_double_button_1=0', 'right_double_button_2=0'
)
if ($Mode -eq 'touchpad') {
    $lines += "${Side}_pad_x=$($X.ToString([cultureinfo]::InvariantCulture))"
    $lines += "${Side}_pad_y=$($Y.ToString([cultureinfo]::InvariantCulture))"
    $lines += "${Side}_pad_touch=$([int]$Touch.IsPresent)"
    $lines += "${Side}_pad_force=$($Force.ToString([cultureinfo]::InvariantCulture))"
}
if ($Mode -eq 'one-button') { $lines += "${Side}_single_button=$([int]$Button1.IsPresent)" }
if ($Mode -eq 'two-buttons') {
    $lines += "${Side}_double_button_1=$([int]$Button1.IsPresent)"
    $lines += "${Side}_double_button_2=$([int]$Button2.IsPresent)"
}
$client = [System.Net.Sockets.UdpClient]::new()
try {
    $payload = [System.Text.Encoding]::ASCII.GetBytes(($lines -join "`n"))
    [void]$client.Send($payload, $payload.Length, '127.0.0.1', 39571)
} finally { $client.Dispose() }
