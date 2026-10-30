# Contributing

Thanks for helping. Windy is small on purpose.

1. Open an issue before large changes.
2. Keep `windy-core` free of OS and UI dependencies. Behaviour comes from `docs/MACCY-SPEC.md`; add a test with every change.
3. Before a pull request, run:
   ```
   cargo fmt --all --check
   cargo clippy --all-targets -- -D warnings
   cargo test -p windy-core -p windy-platform
   npx playwright test
   ```
4. Never log clipboard content. Never add network calls. Keep the CSP in `src-tauri/tauri.conf.json` strict.
5. Conventional commit messages (`feat:`, `fix:`, `docs:`, `test:`, `chore:`).

By contributing you agree your work is licensed under the MIT License.
