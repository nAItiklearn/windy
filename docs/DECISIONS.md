# Decisions log

Short, dated records of project decisions. Architecture-level decisions live in `ADR-*.md`.

## 2026-10-30: Simplifications to ADR-001 (v0.1)

- **UI is plain JavaScript modules, not Svelte.** No bundler, no build step, nothing to break in CI; `ui/` is served as-is by Tauri (`frontendDist`). Same demo-mode idea as the ADR: outside Tauri `ui/api.js` swaps in an in-memory mock, so Playwright tests the real UI code. Revisit if the UI grows.
- **No `tauri-specta` bindings.** The command surface is 12 functions wrapped by one module (`ui/api.js`); a renamed Rust command fails at the first UI call and in the e2e demo only if the mock drifts. Accepted risk for v0.1.
- **`withGlobalTauri` is true** (needed because there is no bundler to import `@tauri-apps/api`). Mitigations: strict CSP, no remote content, command label guard, `core:event:default` as the only capability.
- **Platform layer = `arboard` + `enigo` + `windows-sys`, one crate.** Windows has native sequence-number change detection, foreground-exe lookup and the password-manager exclusion format. macOS and Linux use content-hash polling and report no foreground app yet, so app-based ignore rules do nothing there. Linux is X11 (and XWayland) only.
- **Website is static HTML/CSS/JS (no Astro).** Same reasoning: no build step, deployable as-is with `vercel.json` (`outputDirectory: site`). Download links are driven by `site/downloads.json`.
- **Not in v0.1:** launch at login, preview slide-out, item editor, hold-to-cycle hotkey, pin drag-reorder, window/status-item popup positions, encryption at rest.

## 2026-10-30: Build toolchain is GNU/MinGW, not MSVC

**Context.** Visual Studio Build Tools cannot be installed on the dev PC: the Windows Installer fails with error 8012 on every attempt. The Visual Studio installer is not to be touched again. This supersedes the "Visual Studio Build Tools C++ workload" step in `PROGRESS.md` and `TOOLING.md`.

**Decision.**
- Rust toolchain: `stable-x86_64-pc-windows-gnu`.
- C toolchain: WinLibs MinGW (gcc 16.2, UCRT, POSIX threads), installed with `winget` (`BrechtSanders.WinLibs.POSIX.UCRT`). Its `mingw64\bin` folder goes first in `PATH`:
  `C:\Users\Lenovo\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin`
- The old `C:\MinGW\bin\gcc` is 32-bit and must never be used.
- Before any build, `gcc -dumpmachine` must print `x86_64-w64-mingw32`. If it does not, prepend the folder above to `PATH` in that shell command.
- `CARGO_TARGET_DIR=D:\windy-target` (build output stays outside the repo).
- The toolchain is verified with a tiny `cargo build` before real work starts.

**Fallback if the GNU build of Tauri fails locally.**
- Keep developing and testing `windy-core` (no OS dependencies, builds fine on GNU) and the Svelte UI in demo mode (`npm run demo`) locally.
- Use GitHub Actions `windows-latest` (MSVC) to compile and test the full Tauri app and to produce the installers. Local Tauri builds are a convenience, not a requirement.

**Result (verified 2026-10-30).** The GNU build of Tauri works locally: `cargo build -p windy` succeeds and the app runs on this PC (tray, hotkey, clipboard capture into SQLite). The CI fallback is still used for installers (MSVC on `windows-latest`, plus macOS and Linux).

**Consequences.**
- The ADR-001 measurement gate (popup latency, idle RAM) needs a real Tauri build. If the app cannot be built locally, the gate is measured on a CI-built artifact, and the numbers are recorded in `docs/PERF.md` only once measured.
- Installers come from CI (MSVC), not from the local GNU build.
