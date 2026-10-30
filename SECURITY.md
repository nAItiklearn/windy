# Security policy

Windy handles clipboard content, which often contains secrets. Reports are taken seriously.

**Report privately** via GitHub: *Security* tab, *Report a vulnerability*
(<https://github.com/nAItiklearn/windy/security/advisories/new>). Please do not open a public issue for
vulnerabilities. Expect an acknowledgement within a week.

In scope: leaking clipboard history, storing items that should be ignored, privilege or sandbox escapes from the
webview, installer tampering. Out of scope: attacks requiring an already-compromised user account (the history
database is readable by that account by design).

Design notes and the latest review are in `docs/SECURITY-REVIEW.md`. Early releases are unsigned; verify downloads
against `SHA256SUMS.txt`.
