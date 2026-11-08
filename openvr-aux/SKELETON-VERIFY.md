# Hardware verification (Virtual Desktop first)

This is a development build, not an installed release. It changes the active qptp driver registration
when launched, using the app's normal registration behavior. Keep the installed version for rollback.
Do not register both qptp packages simultaneously.

1. Close QPTP and SteamVR. Keep the Quest/Virtual Desktop connection available.
2. Build the driver and CMake tests, then `npm.cmd run tauri -- build --no-bundle` from the repository root.
3. Start the new app with a unique raw sensor trace:

```powershell
Set-Location C:\Users\lateir\Desktop\qptp
$env:QPTP_SENSOR_TRACE = Join-Path $env:TEMP ('qptp-sensors-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.csv')
Start-Process .\src-tauri\target\release\qptp.exe
Write-Host $env:QPTP_SENSOR_TRACE
```

4. Select Fusion and enable Finger tracking (off by default). Confirm Quest Pro if Virtual Desktop identifies it as Quest 3.
5. Start SteamVR. Enable qptp under Startup/Shutdown → Manage Add-ons if disabled; restart SteamVR if changed.
   The existing Fusion mode and Quest Pro confirmation still require a restart; Finger tracking itself does not.
6. Look for `Skeleton hooks installed`, `Fusion adopted`, and each hand's `using native component` or
   `create partial component result=0` in `vrserver.txt`. The UI should show active for both hands.
7. Run `powershell -ExecutionPolicy Bypass -File .\openvr-aux\verify-skeleton.ps1`.
8. Hold each state for about five seconds, doing the same on both hands: index lifted; index resting
   on the trigger; trigger fully pressed; index slid to each side; thumb at center and each pad corner;
   thumb lifted. Repeat with Finger tracking off and then on.
9. Repeat thumb movement in Button/2 buttons and Position/Joystick/Swipe/Mouse styles. Mouse style
   intentionally moves the Windows cursor; the physical thumb mapping should remain the same.
10. Disconnect one controller and reconnect; interrupt the module connection and restore it. No added
    finger pose should remain after 500 ms without fresh samples. Existing/native animation is the fallback.

The two timestamped CSVs are in `%TEMP%`: `qptp-sensors-*` (pre-clamp hardware values) and
`qptp-skeleton-*` (application-visible action activity, curls, and bone API errors in both motion ranges).
The client needs active actions for both hands and zero bone API errors. CSV success does not prove
physical wrist/finger alignment: also inspect a skeletal-input application's rendered hands.

## OpenXR and regression checks

Use SteamVR as the OpenXR runtime. In a hand-joint inspection application check `XR_EXT_hand_tracking`
availability, `xrLocateHandJointsEXT`, `isActive`, and orientation/position validity for both hands while
holding controllers. Check interaction-profile and session-state changes after reconnecting. Record
runtime/application versions; an OpenVR action test alone does not verify the OpenXR bridge.

Also verify ordinary trigger/grip/touch, dashboard scrolling, haptics, bindings, and Classic mode.
Steam Link and Quest Link/Air Link require a separate run of this checklist before claiming support.

## Rollback

Close SteamVR and the development QPTP app. Launch the previously installed QPTP application; it
restores registration to its own driver directory. Start SteamVR again. Saved Finger tracking is ignored
by older versions; Fusion/Classic and other saved settings remain ordinary app settings.

## Sensor mapping and remaining transport coverage

The initial mapping treats proximity/slide as hardware-normalized 0–1 values. The raw CSV must verify
their polarity/range and the distinction between a lifted finger and a contact at the end of the slide.
Recorded initial samples are in `tests/fixtures/quest_pro_samples.json`. Resting and fully pressed triggers both saturate proximity at 1, so ordinary trigger input supplies the remaining curl. If further measurements contradict the normalized scale, update shared mapping constants and recorded regression
fixtures. There is no user calibration wizard for these sensors. The synthesized fallback hand's wrist
offset still needs a physical fit check on a runtime without a native skeletal component. The native-baseline path was verified through Virtual Desktop.

## Read-only driver diagnostics

The adopted controller's `DriverDebugRequest("qptp_skeleton")` returns source ownership, native update
count, published frame count, sensor freshness, and the latest WithoutController index proximal
quaternion. `skeleton_check` prints these alongside the received quaternion. This detects competing
updates even when `GetSkeletalActionData` continues reporting an active action.

## Verified release behavior (2026-11-07)

- Mapping revision: `index-contact-thumb-transverse-inverted-v8`. The user confirmed correct
  thumb direction/range, trigger slide direction, lifted-index behavior and no flicker on both
  hands in Virtual Desktop + VRChat. Ordinary trigger/grip, buttons, scrolling and vibration work.
- Finger tracking is off by default when the setting is absent; explicit saved choices persist.
  Fusion defaults on for new installations, while existing driver selections are preserved.
  The label, tooltip and statuses are localized in all 12 interface languages.
- Rust tests: 29 pass. CMake skeleton/UDP tests pass, including lifecycle, cached native updates,
  distinct interface instances, validated _004-to-_005 forwarding, fixed bone lengths, mapping,
  toggle/freshness fallback, and the runtime/state lock inversion regression.
- Virtual Desktop uses IVRDriverInput_005. All native writers must be intercepted to avoid
  alternating poses. Runtime publication takes place outside the QPTP state lock to avoid the
  lock inversion that previously caused SteamVR's watchdog to terminate vrserver.
- Live off/on switching restores native animation and re-enables QPTP animation without restart
  or flicker. The VRChat-manifest probe returned active actions and zero bone API errors in both
  motion ranges; sampled index quaternions matched the driver output.
- Left-controller off/on cleared its finger validity while the right stayed valid, then both
  recovered without restarting SteamVR. Virtual Desktop kept its native left SteamVR device
  registered while the hardware controller was off.
- Stopping/restarting QPTP restored native animation on lost reports and resumed fresh tracking
  automatically. These snapshots establish fallback/recovery; automated tests cover timeout bounds.
- OpenXR verification through SteamVR/OpenXR + Virtual Desktop passed: the session reached
  FOCUSED and both hands supplied 26 valid positions/orientations and tracked positions with
  successful locate calls. This verifies API delivery; rendered physical fit was checked in VRChat.
- Steam Link and Quest Link/Air Link, synthesized-fallback physical wrist fit, and OpenXR
  reconnect/profile-change events remain unverified. No support claim is based solely on mocks.

Local session evidence (not included in the repository): `%TEMP%/qptp-skeleton-vrchat-lockfix.csv`,
`qptp-skeleton-left-disconnected.csv`, `qptp-skeleton-left-reconnected.csv`,
`qptp-skeleton-app-disconnected.csv`, `qptp-skeleton-app-reconnected.csv`, and `qptp-openxr-lockfix.txt`.

## Virtual Desktop re-entry regression

A late transport hook on the shared callback could be reached again by a saved _004 thunk that
dynamically dispatched into the patched _005 table, causing a recursive mutex-lock failure.
Validated forwarding wrappers now dispatch directly to the snapshotted inner method with the actual
inner instance. The regression test installs a late inline scalar hook, forwards a nested update under
a nonrecursive transport mutex, and verifies that the transport is not entered twice. A negative-control
build without the forwarding bypass fails this test. The diagnostic log identifies the corrected path:
`Native input forwarding bypass active (Virtual Desktop re-entry fix)`.
