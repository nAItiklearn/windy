# Security review

Reviewed: 2026-10-30, by the author's tooling, against the code at the time. Not an independent audit.

## Threat model

Windy stores clipboard history, which may include passwords and tokens. Assets: the history database and the
live clipboard. Adversaries considered: other local processes running as other users, web content reaching the
webview, malicious clipboard content (pasted into the UI), a tampered download.

Out of scope: malware already running as the same user (it can read the clipboard directly).

## Findings

| # | Area | Finding | Status |
|---|---|---|---|
| 1 | Webview | CSP is `default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data: blob:; connect-src ipc: http://ipc.localhost`. No remote content, no inline script. | Done |
| 2 | Webview | UI builds rows with `textContent`/`createElement`; clipboard text is never assigned to `innerHTML`. Verified by grep (no `innerHTML` in `ui/`). | Done |
| 3 | IPC | Every command checks the calling window label (`popup` or `settings`). The capability file grants only `core:event:default`; no fs, shell, http or opener plugin exists. | Done |
| 4 | Logging | Errors are logged without clipboard content (`eprintln!` of error values only). No logging of items. | Done; add a CI grep guard later |
| 5 | Storage | History is in `%LOCALAPPDATA%\app.windy.clipboard\history.sqlite` (per-user). Unix creation sets mode 0600. **No encryption at rest and no explicit Windows ACL yet.** | Open: documented limitation |
| 6 | Sensitive copies | Windows `ExcludeClipboardContentFromMonitorProcessing` is honoured; ignored apps, regexes and custom types are configurable. `CanIncludeInClipboardHistory=0` alone is not treated as sensitive (Microsoft says it affects Windows history/sync only). | Done |
| 7 | Regex | User-supplied regexes use the `regex` crate (linear time, size-limited in search): no catastrophic backtracking. | Done |
| 8 | Network | The app makes no network requests; there is no updater. | Done |
| 9 | Paste injection | Paste sends `Ctrl+V` to the focused window 150 ms after hiding the popup. A focus change in that window could paste into another app. | Open: inherent to paste automation |
| 10 | Supply chain | `cargo audit` and `npm audit` run in CI (`ci.yml`). Results at review time are recorded below. | See below |
| 11 | Releases | Unsigned installers; SHA256SUMS attached to releases. | Open: needs a code-signing certificate |

## Audit results at review time

Filled in by the commands run during the review (see PROGRESS.md for the exact output summary).
