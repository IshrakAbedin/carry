# STATE — last updated 2026-09-21 (after T006)

- **Current task:** T007 — E2E hardening with a real Conan dependency (fmt) (pending, not started)
- **Last completed:** T006 — `carry add <pkg>[@<version>]` / `carry remove <pkg>` (toml_edit round-trip, comments + spacing survive)
- **One-line state:** All MVP commands work: `new`/`init`, `build [--release]`, `add`/`remove` (format-preserving CARRY.toml edits), `run [args]`, `test`, `clean`; only T007 (real-dep E2E) and T008 (polish) remain.
- **Blockers:** none
- **Open issues:** I001 — conan writes `CMakeUserPresets.json` into the project root, which the scaffold `.gitignore` doesn't cover (fix belongs to template.rs; see ISSUES.md).
- **Baseline checks (2026-09-21, current checkout):** `cargo build` ✅ zero warnings · `cargo clippy --all-targets` ✅ zero warnings · `cargo test` ✅ 28 tests pass
- **Environment:** cargo 1.98.1 · cmake 4.4.3 · conan 2.32.0 · g++ 16.2.1 · ninja 1.13.2 · Conan default profile present · **Conan Center network access VERIFIED working (2026-09-21: fmt/10.2.1 recipe downloaded from conancenter, binary built via `--build=missing`)** — T007's network caveat is resolved.
- **Git:** no global/system git identity on this machine; repo-local identity set to `carry-dev <carry-dev@local>` (matches T000 author). End each task with a local commit, one per task.
- **Deps so far:** clap 4.6.7 (derive), anyhow 1.0.104, serde 1.0.229 (derive), toml 1.1.6, toml_edit 0.25.15; dev: tempfile 3.27.0.

## Resume instructions

Next session: follow BOOTSTRAP.md's protocol and execute **T007**
(`tasks/T007-e2e-real-dep.md` or similar — check TASKS.md). Read
`results/T006.md` (hints section) first: Conan Center is reachable and
`fmt/10.2.1` is now in the local conan cache (Debug, built from source); T007
should use fmt **in code** (`#include <fmt/...>`, `fmt::fmt` link) to exercise
the generated `find_package`/`target_link_libraries` end to end. Known shape:
`manifest.rs` holds `parse_dep_spec` (split on last `@`),
`Manifest::add_dep`/`remove_dep` (path-taking, print change + `run: carry
build`, no write on same version, empty `[dependencies]` left in place after
remove); `build.rs` has `run(cmd, args, cwd, verbose)` +
`build_with(manifest, root, release, verbose)` + `run_cmd`/`test_cmd`/
`clean_cmd`; `run` propagates the child's numeric exit code (101 only on
signal death); `test` failure = carry error (exit 1) in the tool-failure
format; `clean` refuses a `build` symlink pointing outside the project.
Benign warning every configure (conan profile gnu20 vs our CMAKE_CXX_STANDARD
23) — known, see results/T004.md.
