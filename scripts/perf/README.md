# Performance and screenshot harness (Windows only)

`windy-perf.ps1` drives the **real** `windy.exe` and produces the numbers in `docs/PERF.md` and the images in
`docs/screenshots/`. It needs Windows PowerShell 5.1 or later and a release build:

```powershell
cargo build -p windy --release
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/perf/windy-perf.ps1 -Mode startup-ram
```

By default it uses `%CARGO_TARGET_DIR%\release\windy.exe` (or `target\release\windy.exe`) and writes to
`test-results\perf\` (git-ignored). Override with `-Exe` and `-OutDir`. Use forward slashes if you start it from Git Bash.

## Read this first: it takes over the machine

While a mode runs it injects keystrokes (`Ctrl+Alt+V` and others), moves the mouse and clicks, shows a coloured
backdrop window, and in `screenshots` mode replaces the clipboard text with synthetic strings (your own clipboard text is
kept in memory, never printed, and put back afterwards; only plain text is restored). **Do not use the PC while it runs.**
It refuses to run if `windy.exe` is already running, never sends keys unless the popup is the foreground window, and never
clicks unless the popup is visible and focused.

## Modes

| Mode | What it does | Output |
|---|---|---|
| `startup-ram` | 8 launches for start-up times, then one long-lived instance: idle memory (private and total working set per process) and CPU after `-IdleSeconds` (default 60), then `-Cycles` (default 30) popup open/close cycles and memory again | `startup-ram.json` |
| `latency` | `-Trials` (default 40) hotkey presses. Times hotkey to window visible, and hotkey to the first changed pixel at the popup centre. Parks a magenta backdrop exactly behind the popup, and discards a trial if the pointer moved | `latency.json`, `first-frames/*.png` |
| `screenshots` | Feeds ten synthetic strings through the clipboard, then captures the popup (items, selected, pinned, search) and the settings window with `PrintWindow`, and reports how long the settings page takes to show content. Ends with the app's own "Clear all" | `screenshots/*.png` |
| `debug-probe` | Starts the app with a localhost-only WebView2 debugging port and reports each page's real state (URL, readiness, console errors) via `cdp-probe.cjs`. Needs `npm install` (uses `playwright-core`) | printed |

`screenshots` capture with `PrintWindow`, which asks the window to render itself, so another window in front cannot appear in
an image. Plain screen grabs do, which is why `latency` only uses them against its own backdrop.

## A populated history

The app keeps at most 999 items. `tools/seed` writes a synthetic database of that size (and a matching `settings.json`):

```powershell
cargo run --release --manifest-path tools/seed/Cargo.toml -- <some-folder> 999
```

It never touches `%LOCALAPPDATA%\app.windy.clipboard`. To measure with it, quit the app and copy the two files into that
folder yourself. That **replaces any real history** there, so back it up first or use a throwaway Windows profile.

## What the numbers mean (and don't)

See `docs/PERF.md`. In short: one machine, hotkey injected with `keybd_event` (the real `RegisterHotKey` path, but no
physical keyboard), "first paint" is the first pixel change at the popup centre, and the default hotkey `Ctrl+Alt+V` is
assumed. Treat results as a first data point, not a benchmark.

## Gotcha for editing the script

PowerShell variable names are case-insensitive: `$H` (a height) silently overwrote `$h` (a window handle) while this was
being developed and made every later call use a garbage handle. Use distinct names such as `$hwnd` and `$popH`.
