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
| 10 | Supply chain | `cargo audit` and `npm audit` run in CI (`ci.yml`). Results at review time are recorded below. | Done: no vulnerabilities; 5 transitive warnings, see below |
| 11 | Releases | Unsigned installers; SHA256SUMS attached to releases. | Open: needs a code-signing certificate |

## Audit results at review time

Run on 2026-10-30 against commit `27953bf` (branch `feat/bootstrap`), on a Windows dev machine. Tools:
cargo-audit 0.22.2 (RustSec advisory database, 1386 advisories at the time of the run), npm 11.6.2.

Summary: no known vulnerabilities in either ecosystem. `cargo audit` reports 5 warnings (4 unmaintained, 1 unsound).
All five are transitive dependencies of `tauri` and `tauri-plugin-single-instance`; none is a direct dependency of
this repo, so clearing them depends on upstream releases. `cargo audit` exits 0 on warnings, so CI's `audit` job
passes; the warnings are not suppressed or ignored anywhere.

| Advisory | Crate | Kind | Pulled in by |
|---|---|---|---|
| RUSTSEC-2024-0370 | `proc-macro-error` 1.0.4 | unmaintained | `glib-macros` (tauri GTK stack) |
| RUSTSEC-2024-0429 | `glib` 0.18.5 | unsound (`VariantStrIter`) | `gtk`, `atk` (tauri GTK stack) |
| RUSTSEC-2026-0371 | `zbus_names` 4.3.5 | unmaintained | `zbus` (`tauri-plugin-single-instance`) |
| RUSTSEC-2026-0370 | `zvariant` 5.15.1 | unmaintained | `zbus` (`tauri-plugin-single-instance`) |
| RUSTSEC-2026-0372 | `zvariant_derive` 5.15.1 | unmaintained | `zvariant` (`tauri-plugin-single-instance`) |

The "Pulled in by" column comes from `cargo tree --target all -e normal,build -i <crate>`. Which target platforms
actually compile these crates was not checked.

### `cargo audit` (exit code 0)

```
    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 1386 security advisories (from C:\Users\Lenovo\.cargo\advisory-db)
    Updating crates.io index
    Scanning Cargo.lock for vulnerabilities (512 crate dependencies)
Crate:     proc-macro-error
Version:   1.0.4
Warning:   unmaintained
Title:     proc-macro-error is unmaintained
Date:      2024-09-01
ID:        RUSTSEC-2024-0370
URL:       https://rustsec.org/advisories/RUSTSEC-2024-0370

Crate:     zbus_names
Version:   4.3.5
Warning:   unmaintained
Title:     `zbus_names` is unmaintained
Date:      2026-10-16
ID:        RUSTSEC-2026-0371
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0371

Crate:     zvariant
Version:   5.15.1
Warning:   unmaintained
Title:     `zvariant` is unmaintained
Date:      2026-10-16
ID:        RUSTSEC-2026-0370
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0370

Crate:     zvariant_derive
Version:   5.15.1
Warning:   unmaintained
Title:     `zvariant_derive` is unmaintained
Date:      2026-10-16
ID:        RUSTSEC-2026-0372
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0372

Crate:     glib
Version:   0.18.5
Warning:   unsound
Title:     Unsoundness in `Iterator` and `DoubleEndedIterator` impls for `glib::VariantStrIter`
Date:      2024-03-30
ID:        RUSTSEC-2024-0429
URL:       https://rustsec.org/advisories/RUSTSEC-2024-0429

warning: 5 allowed warnings found
```

### `npm audit --omit=dev` (what CI runs; exit code 0)

```
found 0 vulnerabilities
```

### `npm audit` (including devDependencies; exit code 0)

```
found 0 vulnerabilities
```
