# Windy progress

Last updated: 2026-10-30. Branch: `feat/bootstrap` (local only, nothing pushed).

## Done (with evidence)
- Repo initialised locally, `.gitignore` (`.env`, `target/`, `node_modules/`, `reference/`), `.env.example` with empty Snowflake variables.
- Reference screenshots moved to `reference/` (gitignored, never committed).
- Maccy cloned shallow to `D:\windy-scratch\Maccy` (outside the repo). Read: Clipboard, Search, History, HistoryItem, HistoryItemDecorator, Storage, Sorter, PopupPosition, Popup, FloatingPanel, KeyChord, HistoryItemAction, AppState, Footer, AppDelegate, all settings panes, pasteboard type constants, Defaults keys, test names.
- `docs/MACCY-SPEC.md` written from that source (sections 1-12).
- `docs/PORTING-MAP.md` written; Windows clipboard flags, Wayland data-control support and the GlobalShortcuts portal were checked against live docs on 2026-10-30.
- `docs/ADR-001-stack.md`: Tauri 2 + Rust core + Svelte, with a measurement gate.
- Nine specialist agents in `.claude/agents/` (architect, backend, frontend, platform-engineer, debugger, security-reviewer, web-designer, devops-release, qa-tester) with exclusive file ownership.

## Blocked (needs you)
1. **No Rust toolchain.** `cargo`/`rustc`/`rustup` are not installed. Tauri on Windows also needs the MSVC C++ Build Tools (several GB) and WebView2. Installing these changes your machine and needs admin approval; I have not done it. Say "install the toolchain" and I will use `winget` (rustup + Visual Studio Build Tools C++ workload), or tell me if you already have another route.
2. **No `gh` CLI and no push credentials checked.** `git ls-remote` shows `nAItiklearn/windy` exists and is empty. Pushing the branch, tags, releases and any deploy remain yours to approve.
3. **Snowflake Cortex backend for agents.** `SNOWFLAKE_PAT` is set in the environment (value not read or printed); `SNOWFLAKE_ACCOUNT` and `SNOWFLAKE_CORTEX_MODEL` are not. Claude Code subagents speak the Anthropic Messages API; Cortex REST is a different API, so routing them needs a small translating proxy (a local `ANTHROPIC_BASE_URL` endpoint). I need the account identifier and model name, and a decision on whether I should build that proxy. Until then the agents run on the session's own model.
4. **`rea` MCP.** Source inspected (v6.8.0, MIT, `bin` = `scripts/rea.mjs`; the `prepare` hook only runs for installs from the repo). Its documented install is `npx rea-agents setup`, which interactively edits your agent configuration. Please run it yourself: type `! npx rea-agents setup` (from a neutral directory, not from `D:\windy`). Maccy is plain Swift source, so direct reading was sufficient for Phase 1.
5. **Shell sandbox intermittently unavailable** during this session, which blocked `git commit`. The files are on disk but **uncommitted**; first action next session is a commit.

## In progress
- Nothing running.

## Next (in order, once unblocked)
1. Commit docs. 2. Install toolchain. 3. Scaffold workspace (`windy-core`, `windy-platform`, `src-tauri`, `ui`). 4. Core with tests ported from Maccy. 5. Demo-mode UI + Playwright. 6. Windows platform layer (this machine). 7. Measure the ADR gate. 8. macOS/Linux layers (cannot be run here; CI only). 9. Security review, website, ship workflows.

## Decisions
- Tauri 2 stack (ADR-001), gated on measured RAM/latency.
- Not ported: paste stack (disabled in Maccy: `multiSelectionEnabled = false`), App Intents, Vision OCR, Sparkle auto-update.
- `CanUploadToCloudClipboard=0` alone does not hide an item from local history (Microsoft docs say it only affects sync).

## Open questions
- Ship encryption-at-rest in v1 (adds SQLCipher and a perf cost) or defer?
- Per-app paste key (terminals using Ctrl+Shift+V) is not in Maccy; add or document as a limitation?
- Which Linux distros for CI/manual matrix beyond Ubuntu and Fedora?
- macOS and Linux builds can only be produced and tested in CI or on other hardware. Do you have a Mac and a Linux machine for the manual matrix?
