# Windy progress

Last updated: 2026-10-30. Written on branch `docs/ram-decision-and-perf-harness`, which starts from `main` at `ad578f7`.

## State

`main` is GitHub's default branch and holds everything: `windy-core` (history, search, storage, settings), `windy-platform`
(Windows is real; macOS and Linux are compiled in CI only), the Tauri shell, the popup and settings UI, the website, the CI and
release workflows, the security review with `cargo audit` / `npm audit` results, the performance write-up (`PERF.md`) and
screenshots, and Copilot's website-test fix. `feat/bootstrap` is deleted, locally and on origin.

Checked on this Windows PC with a release build: clipboard capture, the global hotkey, popup, search, pin, delete, clear, the
tray, and the settings window. The full Playwright suite (16 tests: 11 popup, 5 website) passes locally on `main` as merged.

ADR-001 gate: popup latency and idle CPU pass. Idle RAM was 103.4 MB against the original 80 MB budget; the project owner
chose to keep the pre-warmed popup and raise the budget to **120 MB** (`DECISIONS.md`, with a dated note in `ADR-001-stack.md`).

## Merged this session

Merged locally and pushed straight to `main` (`gh` is not installed, so no pull requests were opened for them):

- `fix/settings-window-blank`: the settings window opened blank (webview stuck on `about:blank`); the command that opens it
  is now async.
- `fix/clear-shortcut`: `Alt+Ctrl+Backspace` could never clear; adds a regression test.
- `docs/perf-and-screenshots`: `PERF.md`, `docs/screenshots/`, the gate result.

Their branches still exist on origin, along with `feat/security-audit` and `copilot/fix-failing-github-actions-job`. All are
merged and can be deleted.

## Tools in the repo

- `scripts/perf/windy-perf.ps1` (Windows): start-up and memory, hotkey-to-popup latency, screenshots, and a WebView2
  debug-port probe, all against the real app. It takes over keyboard, mouse and clipboard while it runs; read
  `scripts/perf/README.md` first.
- `tools/seed`: writes a synthetic 999-item `history.sqlite` and `settings.json`. Not part of the app workspace or CI.

## Needs you

1. **Check CI.** I cannot read GitHub Actions from here. Pushes to `main` run `ci.yml`; please look at the result for the
   three merges.
2. **A 999-item measurement** needs the seeded database copied into `%LOCALAPPDATA%\app.windy.clipboard`, which replaces any
   real history there. The auto-mode classifier refused to let me write there, and the README explains the manual steps.
3. **`gh`.** `winget install GitHub.cli` failed on this machine with an empty log.
4. **Merge `fix/favicon-500`.** The `500` logged when a page loads was the implicit `GET /favicon.ico`. Neither page declared an
   icon, and Tauri answers any path it cannot find with a 500 because `ui/` has no `index.html`. It happened on the release
   build too (not dev-only), on both the popup and settings pages, and had no user-visible effect. The branch adds an inline
   `<link rel="icon" href="data:,">` to both pages; the probe shows no favicon request and no console error afterwards.

## Not done

- No tag, no GitHub Release, no deploy, and no installer has been built. The release workflow only builds installers on a
  manual run and publishes only for a `v*` tag.
- Search keystroke latency, and anything with a populated history.
- macOS and Linux have never run. They compile in CI only. The 120 MB RAM budget is set from one Windows machine.

## Carried over from earlier notes (not revisited this session)

- Snowflake Cortex backend for the agents: `SNOWFLAKE_ACCOUNT` and `SNOWFLAKE_CORTEX_MODEL` were not set, and routing the
  agents there needs a small translating proxy. Until then the agents run on the session's own model.
- `rea` MCP: install is `npx rea-agents setup`, interactive, to be run by you from a neutral directory.

## Decisions

- Tauri 2 stack (ADR-001), idle-RAM budget amended to 120 MB (see `DECISIONS.md`).
- Not ported: paste stack (disabled in Maccy: `multiSelectionEnabled = false`), App Intents, Vision OCR, Sparkle auto-update.
- `CanUploadToCloudClipboard=0` alone does not hide an item from local history (Microsoft docs say it only affects sync).

## Open questions

- Ship encryption-at-rest in v1 (adds SQLCipher and a performance cost) or defer?
- Per-app paste key (terminals using Ctrl+Shift+V) is not in Maccy; add it or document it as a limitation?
- Which Linux distros for CI and the manual matrix beyond Ubuntu and Fedora?
- macOS and Linux builds can only be produced and tested in CI or on other hardware. Is there a Mac and a Linux machine for
  the manual matrix?
- The ADR gate asks for a 10,000-item history, but the app caps history at 999. Should the gate wording change?
