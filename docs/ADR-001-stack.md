# ADR-001: Technology stack

Status: **accepted, with a measurement gate** (see "Gate"). Date: 2026-10-30.

## Decision

- **Shell**: Tauri 2 (Rust). One codebase for Windows, macOS, Linux.
- **Core**: Rust crate `windy-core` with zero OS dependencies: history engine, dedupe, rules/ignore engine, search, SQLite storage (`rusqlite`, WAL), settings, typed command layer.
- **Platform**: crate `windy-platform` with one trait `Platform` and three implementations (`windows`, `macos`, `linux`): clipboard listener, clipboard read/write, paste injector, frontmost-app info, autostart, global hotkey, backdrop material. Tray and windows come from Tauri.
- **Search**: `nucleo-matcher` for fuzzy (Maccy's fuzzy is Fuse bitap with threshold 0.7; nucleo's score scale differs, so tests assert match presence and ordering, not raw scores), the `regex` crate for regexp, case-insensitive substring for exact. `mixed` = exact, then regexp, then fuzzy, exactly as Maccy.
- **UI**: Svelte 5 (compiled, almost no runtime), plain CSS with variables, system fonts, system accent color, `prefers-color-scheme`, `prefers-reduced-motion`. Two entry points: `popup`, `settings`. No component kit.
- **Contract**: `specta` + `tauri-specta` generate `bindings.ts` from the Rust command and event types. All handlers are registered through one `collect_commands!` list and the UI imports only the generated functions, so a missing or renamed handler fails `cargo build` or `tsc`. CI fails if `bindings.ts` is stale.
- **Demo mode**: `npm run demo` swaps the generated client for an in-memory mock with realistic fake data (up to 10,000 items). Same Svelte components. Used by Playwright and design review.

## Why not the alternatives

| Option | Verdict |
|---|---|
| Native UI per OS (AppKit, WinUI, GTK) over a shared Rust core | Best feel and RAM, but three UIs, three toolchains, three test stacks. Fails "one codebase". It is the **fallback** if the gate fails. |
| Electron | Large installer and high idle RAM. Contradicts principle 1. |
| Qt / Flutter | Qt: licensing and installer weight. Flutter: no native vibrancy, big runtime. Neither beats Tauri for a two-window utility. |
| egui / Slint | Smallest RAM, but non-native look and weaker screen-reader and IME support. Principle 4 and the accessibility requirement argue against. |

## Known risks and mitigations

1. **Idle RAM under 80 MB.** Webview helper processes (WebView2, WebKitGTK, WKWebView) can cost tens of MB each. Mitigation: the core and tray run without any webview; the popup window is created on demand and destroyed shortly after it closes.
2. **Popup under 50 ms after the hotkey.** A cold webview may be slower than that. Mitigation order: (a) one pre-warmed hidden window if idle RAM still meets budget, (b) a minimal pre-parsed shell, (c) the native-popup fallback below.
3. **Wayland/GNOME**: no clipboard-manager protocol and no cursor position (see `PORTING-MAP.md`). Stack-independent; handled by documented fallbacks.
4. **Linux packaging**: WebKitGTK is a runtime dependency. `.deb`/`.rpm` declare it; the AppImage bundles it (large). Documented.
5. **Windows 10**: WebView2 must be present (ships with Windows 11). The NSIS installer uses the small bootstrapper only when it is missing.

These risks are stated as risks, not as results. Nothing about RAM or latency is claimed until it is measured.

## Gate (must pass before Phase 7)

Measured on the dev machine, release build, 10,000-item history, numbers recorded in `docs/PERF.md`:
popup visible < 50 ms after the hotkey, search keystroke to painted result < 16 ms, idle CPU about 0%, idle RAM (all app processes) < 80 MB.
If risks 1 and 2 cannot both be met with a webview, this ADR is superseded by "native popup + Rust core". `windy-core` and `windy-platform` are reused unchanged
and only the UI layer changes. That is why the core never depends on Tauri.

**Amended 2026-10-30.** The first measurement (`PERF.md`) met the popup and CPU budgets and exceeded the original idle-RAM budget
(103.4 MB against 80 MB), with 99.7 MB of that in the WebView2 processes behind the pre-warmed popup. The project owner chose to keep the
pre-warmed popup and raise the idle-RAM budget to **120 MB** (private working set, all app processes) instead of superseding this ADR.
The other gate figures are unchanged. See `DECISIONS.md`.

## Security baseline

- Strict CSP: `default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data: blob:; connect-src ipc: http://ipc.localhost`. No remote content, no `eval`, no inline script.
- One Tauri capability file per window allowlisting only our commands and the window/event APIs the UI needs. No shell, fs, http or opener plugins reachable from the webview.
- Commands check the calling window label (`popup` or `settings`) and reject anything else. `withGlobalTauri` is false. Devtools are off in release builds.
- History DB is created with owner-only permissions (0600 on Unix; per-user directory under `%LOCALAPPDATA%` with an explicit ACL on Windows).
- Encryption at rest (SQLCipher, key in the OS keychain) is a setting, off by default. Whether it ships in v1 is an open question in `PROGRESS.md` because it adds a native dependency and changes the performance profile.
- Logs never contain clipboard content: a `Redacted<T>` wrapper for content types plus a test that fails on content formatting in log macros.
