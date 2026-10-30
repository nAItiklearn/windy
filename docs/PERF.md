# Performance measurements

Measured 2026-10-30 on one Windows machine, on a release build of `windy.exe`. This is the measurement gate from
`ADR-001-stack.md`. Nothing here is claimed for macOS or Linux, and nothing was measured on other hardware.

## Result against the gate

| Gate (ADR-001) | Budget | Measured | Verdict |
|---|---|---|---|
| Popup visible after the hotkey | < 50 ms | Window visible: median **2.4 ms**, p95 4.9 ms, max 7.3 ms. First painted pixel: median **18.3 ms**, p95 25.1 ms, max 35.0 ms (n = 38, empty history) | Pass, for an empty history only |
| Search keystroke to painted result | < 16 ms | Not measured | Open |
| Idle CPU | about 0% | **0.15%** of one core, summed over all 7 app processes, 30 s window | Pass |
| Idle RAM, all app processes | < 80 MB, amended to **< 120 MB** on 2026-10-30 | **103.4 MB** private working set across 7 processes (113.2 MB after 30 popup open/close cycles). Total working set, which counts shared pages, was 376 MB | **Fail** against 80 MB; passes the amended 120 MB with little headroom |
| 10,000-item history | measured with 10,000 items | Not possible as written: `Settings::clamp` caps `history_size` at 999. The 999-item run was not done (see "Not measured") | Gate precondition does not match the product |

The ADR does not say which memory metric the budget means. Private working set is the figure Task Manager shows as
"Memory"; it fails the original 80 MB budget and passes the amended 120 MB one. The total working set (376 MB) would fail
either, which is why the amended budget is defined on private working set.

### Where the memory goes

Idle, 60 s after launch, popup hidden, empty history:

| Process | Private working set | Working set |
|---|---|---|
| `windy.exe` (Rust core, tray, hotkey, clipboard watcher) | 3.7 MB | 26.2 MB |
| `msedgewebview2.exe` x 6 | 99.7 MB together (31.8, 41.1, 14.8, 6.8, 3.1, 2.1) | 350.2 MB together |
| **Total** | **103.4 MB** | **376.4 MB** |

About 96% of the private memory is the WebView2 runtime that hosts the pre-warmed popup. The Rust process alone is
3.7 MB. The ADR planned "create the popup on demand and destroy it shortly after it closes" with a pre-warmed hidden
window only "if idle RAM still meets budget". The implementation pre-warms (the popup window is built in `setup`), and it
does not meet the budget.

The trade-off, stated as reasoning and not as a measurement: dropping the resident webview would cut idle RAM to
roughly the Rust process, but the popup would then pay a webview start-up on every open. The start-up runs below show a
first webview taking hundreds of milliseconds to a few seconds, which would break the 50 ms budget. The ADR's native-popup
fallback is the option that could meet both.

Decision (2026-10-30, project owner): keep the pre-warmed popup and raise the budget to 120 MB. The other two options were
not taken. See `DECISIONS.md`.

## Start-up

Process start to ready, 8 launches (first launch cold), release build:

| Event | min | median | max |
|---|---|---|---|
| Popup window exists (hidden) | 60.5 ms | 123.9 ms | 551.3 ms (the cold first launch) |
| First hotkey press that opens the popup | 663.2 ms | 1667.8 ms | 2625.6 ms |

The second row is the one that matters to a user. It has about 80 ms granularity (the harness retries the hotkey every
80 ms). The gap between the two rows is the popup's WebView2 being created inside `setup`, before the hotkey is
registered. Start-up time is not part of the ADR gate.

## How it was measured

All of it drives the real `windy.exe`, started from a PowerShell script. That logic now lives in
`scripts/perf/windy-perf.ps1` (see `scripts/perf/README.md`; it takes over the keyboard, mouse and clipboard while it
runs). The tables above were measured with earlier scratch versions of the same code, which were then merged into that one
script. Short re-runs of the merged script on the rebuilt binary agreed (window visible median 2.2 ms, first paint 19 ms,
99.5 MB private working set after a short idle), but the tables were not regenerated with it.

- **Hotkey to popup**: inject `Ctrl+Alt+V` with `keybd_event` (the real `RegisterHotKey` path, but not a physical
  keyboard, so keyboard hardware latency is excluded). Time zero is just before the `V` key-down. "Window visible" is the
  first `IsWindowVisible == true`. "First painted pixel" is the first time the pixel at the popup's centre differs from a
  solid magenta window parked behind it. A frame captured 20 to 40 ms after that first changed pixel already shows the
  complete interface (search field, rows, footer), so there is no long blank phase, but the capture does not prove the
  very first changed pixel was already the final frame.
- **Discarded trials**: the pointer positions the popup, and the machine was in use during the runs. Trials where the popup
  did not land where predicted from the pointer position were discarded (2 of 40).
- **Memory**: `WorkingSetPrivate` and `WorkingSet` per process from `Win32_PerfFormattedData_PerfProc_Process`, over the app's
  process tree plus any WebView2 process using the app's own user-data folder.
- **CPU**: summed `TotalProcessorTime` deltas over 30 s.

## Conditions and caveats

- **Machine**: AMD Ryzen 5 5625U (12 logical cores), 13.8 GB RAM, Windows 11 Home build 26300, WebView2 runtime
  156.0.4314.50, one 1920x1080 display at 125% scaling.
- **Binary**: `cargo build -p windy --release` from `main` at `5e590e0`, profile `lto = true`, `codegen-units = 1`,
  `strip = true`, `panic = "abort"`, 8.17 MB. These numbers predate the two fixes on `fix/settings-window-blank` and
  `fix/clear-shortcut`, which do not touch the hot paths measured here.
- **Toolchain**: `stable-x86_64-pc-windows-gnu` with MinGW (see `DECISIONS.md`), not the MSVC build the installers use. The
  linker reported `.rsrc merge failure: multiple non-default manifests`. The app was confirmed per-monitor DPI-aware
  (window DPI 120), so scaling works; other effects of the dropped manifest were not examined.
- **History**: empty during all timing and memory runs.
- **Single machine, single session, in use at the time.** Treat these as a first data point, not a benchmark.

## Not measured

- Search keystroke to painted result (budget 16 ms).
- A populated history. The app caps history at 999 items. `tools/seed` writes a synthetic 999-item database, but it has to
  be copied by hand into `%LOCALAPPDATA%\app.windy.clipboard`, which replaces any real history there (see
  `scripts/perf/README.md`); that was not done. Opening the popup refreshes the list, so latency to a fully painted
  populated list is not covered by the numbers above.
- Idle RAM with the settings window open.
- macOS and Linux.

## Defects found while measuring

Both are fixed on their own branches and are not part of the numbers above.

- The settings window opened blank and never loaded `settings.html` (its webview stayed on `about:blank`) because
  `open_settings_window` was a synchronous command. After the fix the page paints about 70 ms after the window opens. Before
  the fix, "Clear all" also failed once settings had been opened.
- The `Alt+Ctrl+Backspace` "Clear" shortcut could never fire because the `Alt+Backspace` branch matched first.

## Website, local run

Served locally with `npm run site` and audited with Lighthouse in Chrome (no throttling):

| | Accessibility | Best practices | SEO | Agentic browsing |
|---|---|---|---|---|
| Desktop | 100 | 100 | 100 | 100 |
| Mobile | 100 | 100 | 100 | 100 |

53 audits passed and 0 failed in each run. A cold-reload trace gave LCP 223 ms (TTFB 7 ms, render delay 216 ms) and CLS 0.00.
The trace also flagged missing text compression and cache lifetimes; the local `scripts/serve.mjs` sets neither, while
`vercel.json` sets `Cache-Control` headers, so those two flags describe the local server. The deployed site was not
audited and nothing was deployed.

## Screenshots

`docs/screenshots/` holds captures of the real app's own windows (rendered with `PrintWindow`, so no other window can
appear): `popup-items.png`, `popup-pinned.png`, `popup-search.png` and `settings.png`. All content is synthetic text
fed through the clipboard. `settings.png` was taken with the settings fix applied.
