# Windy

A calm clipboard history for Windows, macOS and Linux. Press a shortcut, search, press Enter.
Everything stays on your device: no accounts, no telemetry, no network calls.

Windy follows the behaviour of [Maccy](https://github.com/p0deje/Maccy) (macOS, MIT) but is an independent
project; see `NOTICE`.

## Download

Latest release: <https://github.com/nAItiklearn/windy/releases/latest>

| System | File |
|---|---|
| Windows (developed on Windows 11; Windows 10 should work, not yet verified) | `Windy-windows-x64-setup.exe` |
| macOS (12 or newer should work, not yet verified) | `Windy-macos-universal.dmg` |
| Linux x86-64 (X11 and XWayland) | `Windy-linux-x64.AppImage`, `Windy-linux-x64.deb` |

File names never contain a version, so `/releases/latest/download/<file>` always resolves.
`SHA256SUMS.txt` is attached to every release. Early builds are **not code-signed**: Windows SmartScreen and
macOS Gatekeeper will warn on first launch.

## Use

- **Open**: `Ctrl+Alt+V` (`Cmd+Alt+V` on macOS), or click the tray icon. Change it in Settings.
- **Search** by typing. Modes: exact, fuzzy, regular expression, mixed.
- `Enter` copies. `Alt+Enter` pastes. `Alt+Shift+Enter` pastes without formatting. `Ctrl+1`..`9` picks by number.
- `Alt+P` pins, `Alt+Backspace` deletes, `Esc` closes. `Ctrl+,` opens Settings.
- Copies marked sensitive by password managers (Windows `ExcludeClipboardContentFromMonitorProcessing`) are never stored.
  You can also ignore applications and regular expressions.

## Limits you should know

- Linux: v0.1 supports X11, including XWayland on Wayland desktops. Native Wayland is a known limitation of this
  version (clipboard history needs compositor support; global shortcuts and cursor position are restricted on
  GNOME/Wayland, see `docs/PORTING-MAP.md`). The Linux build has not yet been run on real hardware.
- Paste automation sends `Ctrl+V`. Terminals that paste with `Ctrl+Shift+V` need `Enter` (copy) and a manual paste.
- Popup placement supports "cursor" and "screen centre". macOS and Linux builds are compiled and unit-tested in CI;
  the author has only run the Windows build by hand.

## Build

```
cargo test -p windy-core -p windy-platform
cargo build -p windy            # needs a C toolchain; Windows GNU or MSVC both work
npx @tauri-apps/cli@2 build     # installers
npm install && npx playwright test   # UI in demo mode + website
```

Layout: `crates/windy-core` (history engine, no OS code), `crates/windy-platform` (clipboard, paste, foreground app),
`src-tauri` (shell: tray, hotkey, windows, commands), `ui` (popup and settings, plain JS, demo mode outside Tauri),
`site` (static website). Decisions are logged in `docs/DECISIONS.md`.

## License

MIT. See `LICENSE` and `NOTICE`.
