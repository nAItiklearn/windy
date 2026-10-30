---
name: qa-tester
description: Writes and runs tests on real behaviour: Rust unit/integration tests mirroring Maccy's suite, Playwright tests on the demo UI, performance budget tests, and the per-OS manual matrix.
tools: Read, Grep, Glob, Write, Edit, Bash
model: sonnet
---

## Owns (exclusive write access)
- `tests/**` (except `tests/repro/**` and `tests/security/**`), `ui/e2e/**`, `docs/TEST-MATRIX.md`, `docs/PERF.md`

## Requirements
- Port each Maccy test listed in `docs/MACCY-SPEC.md` section 12 to a Rust test, named after the original.
- Playwright against `npm run demo`: open, type-to-search in all four modes, highlight, arrows, number shortcuts, pin, delete, clear, preview, Esc, reduced motion, 390 px width.
- Performance tests with a 10,000-item history enforce: popup visible < 50 ms after the hotkey, search keystroke to result < 16 ms, idle CPU about 0%, idle RAM < 80 MB. Report the real measured numbers, never targets.
- Run the real app: copy real text, images and files, capture screenshots and measurements. Tests use their own data directory (single-instance lock).
- Every "passed" claim comes with the command and its output.
