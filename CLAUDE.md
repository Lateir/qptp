# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Quest Pro Touch Plus (qptp): a Windows desktop app (Tauri 2 + Vue 3) that shows the extra touch-pad sensors of Quest Pro Touch Pro controllers (X, Y, force) and per-controller status. It receives a stream from [qptp-module](https://github.com/Lateir/qptp-module), which runs on the headset. Gesture bindings and SteamVR integration are the planned next stage. The "Бинды" page is only a placeholder for now.

The UI text, error messages and README are in Russian. Keep new user-facing strings in Russian. The exception is the short status labels on the "Статус" tab (Left/Right, connected/offline, B:), which follow the user's mockup.

## Commands

```powershell
npm install
npm run tauri dev                       # full app (starts Vite on :5173 via beforeDevCommand)
npm run build                           # frontend only: vue-tsc -b type-check + vite build -> dist/
npm run tauri build                     # installer in src-tauri/target/release/bundle
npm run tauri build -- --no-bundle      # release exe without installer
cargo check --manifest-path src-tauri/Cargo.toml   # Rust-only check
```

The project has no test suite and no linter. `vue-tsc -b` (part of `npm run build`) is the type check.

## Architecture

Almost all logic is in two files. Both use a dense, compact style with many statements per line. Match it when editing.

**Backend: `src-tauri/src/lib.rs`**
- A single shared `StreamState` holds a `Mutex<Snapshot>` and an `AtomicU64 generation`. Tauri commands: `get_stream_state`, `set_transport(transport)`.
- The connection is always on: `setup` starts a `worker` thread with the saved transport (`<app_config_dir>/transport`). `set_transport` saves the choice and restarts the worker. Starting a worker bumps `generation`. Every loop and read (`read_exact_checked`, `read_frame`, `worker`) checks whether its captured generation is still current and exits when it is not. This is the cancellation mechanism. Preserve these checks when adding blocking steps.
- All state changes go through `publish()`, which mutates the snapshot and emits the `stream-state` event with the full snapshot. `QPR2` sensor updates are throttled to about one emit every 33 ms.
- `phase` is one of `searching | connecting | connected`. The frontend also sets `error` locally when an invoke fails. The worker retries forever, about once per second.
- Transports:
  - **USB**: runs `adb forward --no-rebind tcp:27182 tcp:27182`. If that fails, it accepts an existing matching forward. It removes the forward on stop only if it created it. Only the bundled adb is used: `<resource_dir>/resources/platform-tools/adb.exe`, shipped via `bundle.resources` in `tauri.conf.json`. It is spawned with `CREATE_NO_WINDOW`. On exit the app runs `adb kill-server` if adb was used.
  - **LAN**: discovery only, with no manual IP. It broadcasts UDP `QPD1` to port 27183 for up to 3 s. The reply is 8 bytes: `QPO1` + u16 LE port + `[1,0]`.
- Stream protocol (TCP, little-endian), ported from `qpro_debug/qpro_stream_receiver.py`:
  - `QPS1` + u32 size (≤4096) + JSON status body.
  - `QPR2` + 28-byte body. Left sensor at bytes 12–19 (u16 x, u16 y, f32 force), right sensor at 20–27.
  - Any other header is an error and triggers a reconnect.
  - Y is inverted in `sensor()` (`255 - raw`), so 0 is the top of the pad. The frontend draws it as is.
- Tray and window lifecycle:
  - Minimizing destroys the `main` window, so WebView2 frees its memory.
  - `ExitRequested` with `code: None` is prevented, so the app keeps running in the tray.
  - Closing the window (X) or choosing "Выход" calls `app.exit(0)`.
  - Left-clicking the tray icon recreates the window from `tauri.conf.json` on a separate thread, because building it inside an event handler deadlocks on Windows.
  - The right-click menu shows L/R dots, updated from `publish()` only when the connected state changes. The update happens after the snapshot lock is released, because menu calls block on the main thread.
  - The frontend has to recover all its state from `get_stream_state` on mount.

**Frontend: `src/App.vue`**
- Layout: a fixed 780×330 window (not resizable, set in `tauri.conf.json`). Touch pads `components/TouchPad.vue` sit on the left and right. The center column has the title, a view switched by the `tab` ref (Режим = USB/LAN choice, Статус = controller cards, Настройки = info) and a pill tab bar. There is no router.
- TouchPad shows X/Y/F normalized to 0..1: x,y are divided by 255, force is already 0..1. The dot grows with force. It uses `mix-blend-mode: difference` on a white fill, so the digits stay readable under it in both themes.
- Theming is pure CSS: tokens on `:root`, overridden in `@media (prefers-color-scheme: dark)` in `src/style.css`. Fonts are bundled via `@fontsource-variable`: Unbounded (display), Manrope (UI), JetBrains Mono (numbers).
- It mirrors the Rust `Snapshot` as a TS type (camelCase via `serde(rename_all)`). If you change the Rust struct, update the TS type too.
- Status JSON is read defensively: it accepts `status.left` or `status.controllers.left`, `battery_percent` or `battery`, and `tracked` or `tracking`, because the module's schema isn't pinned here.
- Tauri calls are guarded with `isTauri()`, so `npm run dev` in a plain browser renders the initial state.
- `src/style.css` is plain CSS. It does not import Tailwind. The shadcn-vue components in `src/components/ui/` (`components.json`) are currently unused. `@` is an alias for `src/`.
