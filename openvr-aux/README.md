# QPTP auxiliary OpenVR driver

The driver adds one controller-class device with a `TrackedControllerRole_Treadmill` hint alongside the normal Quest Pro controllers. It exposes two pads (X, Y, touch, and force) and six mode-specific buttons: 14 input components in total. QPTP sends controller data to the driver over local UDP. The driver's `IInputSource` interface abstracts input reception; the included debug sender sends packets to the same UDP listener.

## Build

Use 64-bit Windows with the Visual Studio 2022 C++ tools. From the repository root, build with CMake:

```powershell
cmake -S .\openvr-aux -B .\openvr-aux\build -G "Visual Studio 17 2022" -A x64
cmake --build .\openvr-aux\build --config Release
```

Alternatively, use the MSVC build script, which does not require a separate CMake installation:

```powershell
powershell -File .\openvr-aux\build-driver.ps1
```

Both methods write `driver_qptp.dll` to `openvr-aux\driver_qptp\bin\win64`. The vendored OpenVR driver header and its license are in `openvr-aux\vendor\openvr`. The driver does not link against `openvr_api.dll`; SteamVR supplies the driver interfaces through `IVRDriverContext`.

## Install

Close SteamVR before changing driver registration or replacing the DLL. Adjust the SteamVR path if needed:

```powershell
$vr = "C:\Program Files (x86)\Steam\steamapps\common\SteamVR"
$driver = (Resolve-Path .\openvr-aux\driver_qptp).Path
& "$vr\bin\win64\vrpathreg.exe" adddriver "$driver"
& "$vr\bin\win64\vrpathreg.exe" show
```

Start SteamVR with the Quest Pro controllers connected. If you previously registered the experimental `qproxy` driver, remove its old registration with `vrpathreg.exe removedriver <path-to-driver_qproxy>`.

The QPTP Windows installer bundles and registers this driver automatically. Its `driver.vrdrivermanifest` sets `alwaysActivate=true`, so SteamVR loads it alongside another headset driver. When installed beside `qptp.exe`, the driver launches the app on SteamVR startup if **Start and close with SteamVR** is enabled (the default). The app exits five seconds after losing the SteamVR connection if it connected during that run. Its single-instance mutex prevents duplicate app instances. If SteamVR was not installed when QPTP was installed, launching the installed app retries driver registration once SteamVR becomes available.

## Verify input and bindings

1. In SteamVR's `logs\vrserver.txt`, look for `[qptp] Driver initialized`, `Device registered`, `Device activated index=...`, `Input component created`, and `Debug UDP listening`.
2. In **SteamVR Settings → Controllers → Manage Controller Bindings**, choose an application and select the additional **QPTP Auxiliary Input** device. Its `/user/treadmill` input source offers the left and right pads and six buttons. The binding editor shows both QPTP and the normal hand controllers; choosing QPTP does not replace the hand controllers.
3. Run the QPTP app with `dev.bat` or `npm run tauri dev`, connect the Quest module over USB or LAN, and select an input mode. **Touchpad** sends X, Y, touch, and force. **Button** sends `left_single_button` and `right_single_button`; **2 buttons** sends the corresponding `double_button_1` and `double_button_2` signals. Button modes zero the pads and inactive button paths; lost module input is also zeroed.
4. Watch `vrserver.txt` for `Input update` entries. These confirm that packets reached the driver and its `IVRDriverInput` components; they do not alone prove that a particular application's action fired. To test the driver without the QPTP app, send a packet with `powershell -File .\openvr-aux\send-debug.ps1 -Mode touchpad -Side left -Touch -X 0.4 -Y -0.2 -Force 0.8`. Values reset two seconds after the last packet.
5. Save a binding for the target application and test its action in that application. QPTP bindings have been confirmed to work with OVR Advanced Settings while the regular Quest Pro controller bindings remain available. Bindings saved for the former `extra_1` and `extra_2` paths must be reassigned to the new mode-specific paths.

The input profile is `driver_qptp/resources/input/qptp_profile.json`. It declares `/pose/raw` and uses `input_bindingui_mode: single_device` with `input_bindingui_right`. The driver reports a connected, stationary identity pose with `poseIsValid=true` and `TrackingResult_Running_OK`; it does not provide real spatial tracking. QPTP bindings worked with this pose in the tested SteamVR setup. The diagnostic `GetControllerRoleForTrackedDeviceIndex` call can still return `0` while QPTP bindings work, despite the treadmill role hint. The device sets a render model name for identification, but has no 3D model; the binding UI uses an SVG.

The UDP listener accepts packets only on `127.0.0.1:39571`. Packets contain `key=value` pairs separated by newlines or commas. Keys are `left_pad_x`, `left_pad_y`, `left_pad_touch`, `left_pad_force`, the corresponding `right_*` pad keys, and `left_single_button`, `left_double_button_1`, `left_double_button_2` with corresponding right-side keys. X and Y are clamped to [-1, 1], force to [0, 1], and any nonzero numeric value is true for a boolean input. The driver acknowledges received packets with `QPTP1`.

## Uninstall a manually registered driver

Close SteamVR, then run:

```powershell
$vr = "C:\Program Files (x86)\Steam\steamapps\common\SteamVR"
$driver = (Resolve-Path .\openvr-aux\driver_qptp).Path
& "$vr\bin\win64\vrpathreg.exe" removedriver "$driver"
```

The driver directory can then be removed. The driver writes property and input API errors through `IVRDriverLog` to `vrserver.txt`. It updates components each frame and logs received input changes at most once per second.
