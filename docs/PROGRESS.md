# Windy progress

Last updated: 2026-10-30. Written on branch `docs/perf-and-screenshots`, which starts from `main` at `5e590e0`.

## State

`main` holds the whole first batch: `windy-core` (history, search, storage, settings), `windy-platform` (Windows is real;
macOS and Linux are compiled in CI only), the Tauri shell, the popup and settings UI, the website, the CI and release
workflows, and the security review with `cargo audit` / `npm audit` results (PR #1).

Checked on this Windows PC with a release build: clipboard capture, the global hotkey, popup, search, pin, delete, clear,
the tray, and the settings window (after the fix below). Measurements and screenshots are in `PERF.md` and
`docs/screenshots/`. The ADR-001 gate **passes on popup latency and idle CPU and fails on idle RAM** (103 MB against 80 MB);
see `DECISIONS.md`.

## Branches waiting for a pull request

`gh` is not installed on this PC, so none of these has a PR yet.

| Branch | What | Compare |
|---|---|---|
| `fix/settings-window-blank` | The settings window opened blank (webview stuck on `about:blank`); the command that opens it is now async | https://github.com/nAItiklearn/windy/compare/main...fix/settings-window-blank?expand=1 |
| `fix/clear-shortcut` | `Alt+Ctrl+Backspace` could never clear (the delete branch matched first); adds a regression test | https://github.com/nAItiklearn/windy/compare/main...fix/clear-shortcut?expand=1 |
| `docs/perf-and-screenshots` | `PERF.md`, screenshots, this file, the gate result in `DECISIONS.md` | https://github.com/nAItiklearn/windy/compare/main...docs/perf-and-screenshots?expand=1 |

## Needs you

1. **Default branch, and do not delete `feat/bootstrap` on origin yet.** GitHub's default branch is still `feat/bootstrap`,
   and Copilot's PR #2 was merged into it, not into `main`. That put commit `b863b61` only on `feat/bootstrap`: a small fix
   to `tests/e2e/site.spec.js` that also stubs `navigator.userAgentData` for the OS-detection test. `main` lacks it, so the
   website test may fail in CI on pull requests from `main`-based branches until it is brought over (a PR from
   `feat/bootstrap` to `main`, or a cherry-pick). Then set the default branch to `main` (Settings, General) and delete
   `feat/bootstrap` on origin. My local copy of that branch is already deleted.
2. **Idle RAM gate.** Keep the pre-warmed popup and amend the 80 MB budget, drop the webview when the popup closes, or
   build the native popup. See `DECISIONS.md`.
3. **A 999-item measurement** needs a seeded database in `%LOCALAPPDATA%\app.windy.clipboard`. The auto-mode classifier
   refused to let me write there, so I did not. The synthetic database is at `D:\windy-scratch\seeddata`.
4. **`gh`.** `winget install GitHub.cli` failed on this machine with an empty log.
5. `origin/copilot/fix-failing-github-actions-job` (Copilot's PR #2, merged into `feat/bootstrap`): I read its diff against
   `main` and it looks reasonable, but I have not run that test.

## Not done

- No tag, no GitHub Release, no deploy, and no installer has been built. The release workflow only builds installers on a
  manual run and publishes only for a `v*` tag.
- Search keystroke latency, and anything with a populated history.
- macOS and Linux have never run. They compile in CI only.

## Carried over from earlier notes (not revisited this session)

- Snowflake Cortex backend for the agents: `SNOWFLAKE_ACCOUNT` and `SNOWFLAKE_CORTEX_MODEL` were not set, and routing the
  agents there needs a small translating proxy. Until then the agents run on the session's own model.
- `rea` MCP: install is `npx rea-agents setup`, interactive, to be run by you from a neutral directory.

## Decisions

- Tauri 2 stack (ADR-001), gated on measured RAM and latency.
- Not ported: paste stack (disabled in Maccy: `multiSelectionEnabled = false`), App Intents, Vision OCR, Sparkle auto-update.
- `CanUploadToCloudClipboard=0` alone does not hide an item from local history (Microsoft docs say it only affects sync).

## Open questions

- Ship encryption-at-rest in v1 (adds SQLCipher and a performance cost) or defer?
- Per-app paste key (terminals using Ctrl+Shift+V) is not in Maccy; add it or document it as a limitation?
- Which Linux distros for CI and the manual matrix beyond Ubuntu and Fedora?
- macOS and Linux builds can only be produced and tested in CI or on other hardware. Is there a Mac and a Linux machine for
  the manual matrix?
