# Quest Pro Touch Plus (qptp)

A Windows app for viewing the additional touch sensors on Quest Pro Touch Pro controllers. It shows the connection state, each controller's status, and live touchpad values (X, Y, and force). The [qptp SteamVR auxiliary driver](openvr-aux/README.md) receives the same live input from this app and exposes it as one additional device alongside the stock Quest Pro controllers.

## Requirements

- Windows 10/11 with WebView2.
- A Quest Pro with [qptp-module](https://github.com/Lateir/qptp-module) installed and running. The module provides the QPR2/QPS1 stream.
- USB: ADB debugging authorized on the Quest. ADB and its DLLs are bundled with the app.
- LAN: the computer and Quest on the same local network, with access to UDP port 27183 and TCP port 27182. The app discovers the Quest automatically by UDP broadcast.

## Run for development

Run `dev.bat` by double-clicking it or from a terminal. It installs dependencies with `npm ci` if needed.

```powershell
.\dev.bat
```

## Build for Windows

Run `build-installer.bat` to build the Windows NSIS installer. It compiles and bundles the qptp OpenVR driver. During installation, the installer registers the driver with SteamVR; during uninstallation, it removes that registration. SteamVR must be installed for registration, and needs a restart to load or unload the driver reliably.

```powershell
.\build-installer.bat
```

The installer is written to `src-tauri/target/release/bundle/nsis`. For a quick build without an installer, run `npm.cmd run tauri -- build --no-bundle`. In PowerShell, use `npm.cmd`: execution policy may block `npm.ps1` when calling `npm`.

## Behavior

The app connects automatically on startup and retries if the Quest is unavailable or the stream disconnects. LAN is selected on the first launch. Choose USB or LAN on the **Settings** tab; the choice is saved. USB uses the bundled ADB and creates a `tcp:27182` port forward. LAN broadcasts the `QPD1` discovery packet to UDP port 27183, then connects to the discovered address. `QPS1` supplies controller connection, battery, and tracking data; `QPR2` updates X, Y, and force. Y = 0 is the top of the touchpad.

Minimizing destroys the window to release WebView2 memory while the app continues running in the system tray. Left-click the tray icon to reopen the window. Right-click to see the L and R controller indicators (green = connected, gray = disconnected) and **Quit**. Closing the window with its X button exits the app.

On the **Mode** tab, choose one mode for both controllers: **Touchpad**, **Button**, or **2 buttons**. In two-button mode, the upper and lower halves of each touchpad act as separate virtual buttons; the selected half stays active until release. The default press threshold is 0.30 and the release threshold is 0.20. The press threshold slider ranges from 0.21 to 1.00.

The **Status** tab shows SteamVR as connected only after the qptp driver acknowledges the app's input packets. The **Settings** tab has a **Start and close with SteamVR** switch (enabled by default). When enabled, SteamVR loads the installed driver at startup, the driver starts the installed app if needed, and the app closes after SteamVR disconnects for five seconds. A manually started app stays open if SteamVR has not connected during that run. When disabled, SteamVR does not start the app and closing SteamVR does not close an app that was started manually. If the app was last minimized to the tray, it starts there again. Opening it from the tray records the visible state for the next launch.

Each press sends a 2 ms haptic pulse to the corresponding controller's thumb zone. Further pulses for that controller are dropped until the module acknowledges completion. A separate slider sets vibration strength from 0% to 100% (20% by default; 0% disables vibration). The mode, press threshold, vibration strength, and transport are saved in the app's user `config.json`. On the first launch after an upgrade, the app can read the previous `transport` file as a migration fallback. On Windows, the app sends the current mode and sensor state to the local qptp SteamVR driver. SteamVR action bindings still need to be assigned per application.

The **Settings** tab also has a button that opens the module repository in the system browser. The app and Tauri installer versions both come from `package.json`.

You can select the interface language in **Settings**. The list contains English, Simplified Chinese, Hindi, Spanish, Arabic, French, Bengali, Brazilian Portuguese, Russian, Indonesian, German, and Japanese, ordered by total speakers. On first launch, the app selects the first supported language in the user's preferred Windows UI languages; if none is supported, it selects English. The choice is saved in `config.json` and is not replaced by the system language on later launches. Arabic text is displayed right to left while the physical left and right controllers retain their positions. The tray menu follows the selected language.

## Protocol and source material

The receiver was ported from `qpro_debug/qpro_stream_receiver.py`. Running Python is not required. The protocol and Quest module are documented in [qptp-module](https://github.com/Lateir/qptp-module). ADB is distributed with the Android Platform Tools `NOTICE.txt`.

When using LAN, connect over a trusted local network: the current module does not authenticate TCP clients.
