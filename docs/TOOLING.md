# Tooling record

Everything installed or enabled for this project, and why. Nothing is enabled without being listed here.

## Installed / cloned (scratch, outside the repo, in `D:\windy-scratch`)
| Item | Source | State | Notes |
|---|---|---|---|
| Maccy | https://github.com/p0deje/Maccy (MIT) | cloned, shallow | Read-only reference. Code is never copied into Windy. |
| rea | https://github.com/morluto/rea (v6.8.0, MIT) | cloned, **not installed** | Package inspected: `bin` -> `scripts/rea.mjs`; `prepare` script runs only on repo installs. Official install is `npx rea-agents setup`, which edits agent config; left for the user to run interactively. |
| ECC | https://github.com/affaan-m/ECC | cloned, shallow | Used for the agent file format (frontmatter `name`, `description`, `tools`, `model` + prompt body). Windy's agents are written fresh. |

## Not yet installed (blocked, see PROGRESS.md)
rustup / Rust stable, Visual Studio Build Tools (C++ workload), WebView2 runtime check, `gh` CLI.

## MCPs
None enabled by this project. Already available in the session: Chrome DevTools (screenshots, Lighthouse, performance traces), which covers the website and UI review needs. Candidates, to be inspected before enabling: Playwright MCP (UI tests; the Playwright test runner itself is a dev dependency and needs no MCP), Context7 (library docs). shadcn/ui and 21st.dev Magic are not needed: the design brief excludes UI kits.

## Model backend
Agents default to the session model. Snowflake Cortex routing is pending account details and a translating proxy (PROGRESS.md, blocker 3). Credentials are read from environment variables only: `SNOWFLAKE_ACCOUNT`, `SNOWFLAKE_PAT`, `SNOWFLAKE_CORTEX_MODEL` (see `.env.example`). `.env` is gitignored.
