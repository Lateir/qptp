# QPTP auxiliary OpenVR driver

The driver adds one controller-class input device, `qptp_input`, with a `TrackedControllerRole_Treadmill` hint alongside the normal Quest Pro controllers. It exposes two pads (X, Y, touch, force), six mode-specific buttons, and three analog sensors per side (stylus, trigger proximity, trigger slide). Two more input-less devices, `qptp_left_battery` and `qptp_right_battery`, only show each controller's battery level in SteamVR. QPTP sends data to the driver over local UDP.

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
2. In **SteamVR Settings → Controllers → Manage Controller Bindings**, choose an application and select the **QPTP Auxiliary Input** device. Its `/user/treadmill` input source offers both pads, six buttons, and the analog sensors. The binding editor shows both QPTP and the normal hand controllers; choosing QPTP does not replace the hand controllers.
3. Run the QPTP app with `dev.bat` or `npm run tauri dev`, connect the Quest module over USB or LAN, and select an input mode. **Touchpad** sends X, Y, touch, and force. **Button** sends `left_single_button` and `right_single_button`; **2 buttons** sends the corresponding `double_button_1` and `double_button_2` signals. Button modes zero the pads and inactive button paths; lost module input is also zeroed.
4. Watch `vrserver.txt` for `Input update` entries. These confirm that packets reached the driver and its `IVRDriverInput` components; they do not alone prove that a particular application's action fired. To test the driver without the QPTP app, send a packet with `powershell -File .\openvr-aux\send-debug.ps1 -Mode touchpad -Side left -Touch -X 0.4 -Y -0.2 -Force 0.8`. Values reset two seconds after the last packet.
5. Save a binding for the target application and test its action in that application. QPTP bindings have been confirmed to work with OVR Advanced Settings while the regular Quest Pro controller bindings remain available. Bindings saved for the former `extra_1` and `extra_2` paths must be reassigned to the new mode-specific paths.

The input profile is `driver_qptp/resources/input/qptp_profile.json`. It declares `/pose/raw` and uses `input_bindingui_mode: single_device`. The driver reports a connected, valid, stationary pose 9001 m above the origin, so applications that enumerate all controllers cannot see or use it; it does not provide real spatial tracking. The pose must stay valid: with `poseIsValid=false` the bindings did not work. Only the left hand, right hand and treadmill roles receive a `/user/...` path that bindings can target, so the input device must keep the treadmill role; an OptOut device receives input components but SteamVR routes none of them to application actions.

The battery devices use the OptOut role with the lowest hand-selection priority and `qptp_battery_profile.json`, which declares only `/pose/raw`. They set `Prop_NeverTracked_Bool` and never report a valid pose, so applications cannot use their position. Each one appears connected only while QPTP reports a battery level for its controller and disconnects two seconds after the last packet.

The UDP listener accepts packets only on `127.0.0.1:39571`. Packets contain `key=value` pairs separated by newlines or commas. Each side supports `{side}_pad_x`, `_pad_y`, `_pad_touch`, `_pad_force`, `_single_button`, `_double_button_1`, `_double_button_2`, `_stylus`, `_trigger_proximity`, `_trigger_slide`, `_battery`, and `_battery_valid`. Battery is a fraction from 0 to 1 and is only reported when valid. X and Y are clamped to [-1, 1]; other analog values to [0, 1]. The driver acknowledges received packets with `QPTP1`.

## Uninstall a manually registered driver

Close SteamVR, then run:

```powershell
$vr = "C:\Program Files (x86)\Steam\steamapps\common\SteamVR"
$driver = (Resolve-Path .\openvr-aux\driver_qptp).Path
& "$vr\bin\win64\vrpathreg.exe" removedriver "$driver"
```

The driver directory can then be removed. The driver writes property and input API errors through `IVRDriverLog` to `vrserver.txt`. It updates components each frame and logs received input changes at most once per second.
