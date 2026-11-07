# QPTP auxiliary OpenVR driver

The driver has two modes. The app writes the selected one to `%APPDATA%\dev.lateir.qptp\steamvr_driver_mode` (`classic` or `fusion`), and the driver reads it once when SteamVR starts.

**Classic** (preserved on updates from Classic) adds one controller-class input device, `qptp_input`, with a `TrackedControllerRole_Treadmill` hint alongside the normal Quest Pro controllers. It exposes two pads (X, Y, touch, click, force), six mode-specific buttons, and three analog sensors per side (stylus, trigger proximity, trigger slide). Two more input-less devices, `qptp_left_battery` and `qptp_right_battery`, only show each controller's battery level in SteamVR.

**Fusion** (beta) adds no devices. In `Init`, it replaces the first entry of the `IVRServerDriverHost` vtable (`TrackedDeviceAdded`) and wraps every controller that another driver creates. When a wrapped controller activates with a `quest_pro` render model, the `{oculus}/input/touch_profile.json` profile and a hand role, Fusion:
- switches it to `{qptp}/input/qptp_quest_pro_profile.json` (controller type `qptp_quest_pro`);
- creates `/input/trackpad/{x,y,touch,force,click}`, `/input/pad_button/click`, `/input/pad_upper/click`, `/input/pad_lower/click`, `/input/stylus/value`, `/input/trigger_proximity/value` and `/input/trigger_slide/value` on it;
- writes the battery level to the controller itself.

Every controller Fusion sees is logged with its render model, profile and role. A Quest 3 render model (`quest_plus`, as Virtual Desktop can report for Quest Pro hardware) is reported to the app as `q3left`/`q3right`. It is adopted only when `%APPDATA%\dev.lateir.qptp\steamvr_fusion_quest3` contains `pro`, written after the user confirms in the app; such controllers also get the `oculus_quest_pro_controller_left/right` render model. Controllers whose profile already starts with `{qftplus}` are left alone and reported as a QFTPlus conflict. Controllers created before qptp's `Init` are not wrapped; reconnecting them fixes that. If patching the vtable fails, the driver registers the Classic devices instead and reports `fallback`. `Cleanup` restores the original vtable entry. The approach, the profile, the remapping file and the dashboard bindings are adapted from [QFTPlus](https://github.com/Yeusepe/QFTPlus) (MIT).

QPTP sends data to the driver over local UDP. The app applies calibration, resting-thumb rejection, the force click and the touchpad style before sending, so the driver forwards final values.

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
4. Watch `vrserver.txt` for `Input update` entries. These confirm that packets reached the driver and its `IVRDriverInput` components; they do not alone prove that a particular application's action fired. To test the driver without the QPTP app, send a packet with `powershell -File .\openvr-aux\send-debug.ps1 -Mode touchpad -Side left -Touch -X 0.4 -Y -0.2 -Force 0.8`. Values reset 500 ms after the last packet; the app sends a heartbeat every 100 ms.
5. Save a binding for the target application and test its action in that application. QPTP bindings have been confirmed to work with OVR Advanced Settings while the regular Quest Pro controller bindings remain available. Bindings saved for the former `extra_1` and `extra_2` paths must be reassigned to the new mode-specific paths.

The input profile is `driver_qptp/resources/input/qptp_profile.json`. It declares `/pose/raw` and uses `input_bindingui_mode: single_device`. The driver reports a connected, valid, stationary pose 9001 m above the origin, so applications that enumerate all controllers cannot see or use it; it does not provide real spatial tracking. The pose must stay valid: with `poseIsValid=false` the bindings did not work. Only the left hand, right hand and treadmill roles receive a `/user/...` path that bindings can target, so the input device must keep the treadmill role; an OptOut device receives input components but SteamVR routes none of them to application actions.

The battery devices use the OptOut role with the lowest hand-selection priority and `qptp_battery_profile.json`, which declares only `/pose/raw`. They set `Prop_NeverTracked_Bool` and never report a valid pose, so applications cannot use their position. Each one appears connected only while QPTP reports a battery level for its controller and disconnects 500 ms after the last packet.

The UDP listener accepts packets only on `127.0.0.1:39571`. Packets contain `key=value` pairs separated by newlines or commas. Each side supports `{side}_pad_x`, `_pad_y`, `_pad_touch`, `_pad_force`, `_pad_click`, `_single_button`, `_double_button_1`, `_double_button_2`, `_stylus`, `_trigger_proximity`, `_trigger_slide`, `_battery`, and `_battery_valid`. Battery is a fraction from 0 to 1 and is only reported when valid. X and Y are clamped to [-1, 1]; other analog values to [0, 1]. The driver acknowledges every packet with `QPTP2 mode=<classic|fusion|fallback> left=<0|1> right=<0|1> q3left=<0|1> q3right=<0|1> conflict=<none|qftplus>`, where `left`/`right` say whether Fusion adopted that controller. The app still accepts the older `QPTP1` acknowledgement as Classic.

## Uninstall a manually registered driver

Close SteamVR, then run:

```powershell
$vr = "C:\Program Files (x86)\Steam\steamapps\common\SteamVR"
$driver = (Resolve-Path .\openvr-aux\driver_qptp).Path
& "$vr\bin\win64\vrpathreg.exe" removedriver "$driver"
```

The driver directory can then be removed. The driver writes property and input API errors through `IVRDriverLog` to `vrserver.txt`. It updates components each frame and logs received input changes at most once per second.

## Finger tracking (development)

Fusion is the default for new installations; existing driver choices are preserved. Fusion supports a live **Finger tracking** switch, disabled by default. It adds thumb-rest position and
index proximity/slide to each adopted hand. Native skeletons are retained as the baseline. If no native
component exists, QPTP creates a synthesized hand animated from ordinary trigger/grip/touch inputs.
Both WithController and WithoutController streams are supplied. Turning the switch off restores the
baseline immediately; Classic mode does not provide this feature.

The physical finger channel is independent of pad mode/style and resting-thumb rejection. QPR4 contact
flags are authoritative. Sensors older than 500 ms are ignored even if the app continues UDP heartbeats.
The UDP extension uses `skeleton_enabled` and per-side `finger_{x,y,contact,proximity,slide,valid,age_ms}`.
Raw proximity/slide are not clamped by the app on this channel. The QPTP2 reply adds `skelleft`/`skelright`
with `active`, `waiting`, `disabled`, `error`, or `unavailable`; absence means an older driver.

The skeletal interceptor covers IVRDriverInput_003/_004 and a validated compatible prefix of _005
on Windows x64. _005 support is enabled only after checking the _004 forwarding thunks and matching implementation tables. MinHook also captures cached skeleton-update entry points. Like Fusion's controller hook, this integration is outside the public driver API and must
be checked against SteamVR/runtime updates. Devices not adopted by Fusion pass through unchanged.

Sensor range/contact checks have been performed on Quest Pro through Virtual Desktop; visual pose fit and multi-runtime regression checks are still in progress.
See [SKELETON-VERIFY.md](SKELETON-VERIFY.md) for exact capture, verification and rollback steps.
The hand geometry uses [Valve's sample](vendor/hand_simulation/README.md); the driver's Valve license
notice is included in `driver_qptp/LICENSE.Valve.txt`.
