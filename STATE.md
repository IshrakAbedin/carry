# STATE — last updated 2026-09-21 (after T005)

- **Current task:** T006 — `carry add` / `carry remove` (toml_edit, format-preserving) (pending, not started)
- **Last completed:** T005 — `carry run` (exec, child exit code propagated), `carry test` (ctest), `carry clean`
- **One-line state:** All daily-driver commands work end-to-end: `build [--release]`, `run [args]` (exit code propagated, `--` separator documented), `test` (ctest, failure = carry error exit 1), `clean` (symlink-safe); only `add`/`remove` stubs remain before T007.
- **Blockers:** none
- **Open issues:** I001 — conan writes `CMakeUserPresets.json` into the project root, which the scaffold `.gitignore` doesn't cover (fix belongs to template.rs; see ISSUES.md).
- **Baseline checks (2026-09-21, current checkout):** `cargo build` ✅ zero warnings · `cargo clippy --all-targets` ✅ zero warnings · `cargo test` ✅ 20 tests pass
- **Environment:** cargo 1.98.1 · cmake 4.4.3 · conan 2.32.0 · g++ 16.2.1 · ninja 1.13.2 · Conan default profile present · Conan Center network access still unverified (T007).
- **Git:** no global/system git identity on this machine; repo-local identity set to `carry-dev <carry-dev@local>` (matches T000 author). End each task with a local commit, one per task.
- **Deps so far:** clap 4.6.7 (derive), anyhow 1.0.104, serde 1.0.229 (derive), toml 1.1.6, toml_edit 0.25.15; dev: tempfile 3.27.0.

## Resume instructions

Next session: follow BOOTSTRAP.md's protocol and execute **T006**
(`tasks/T006-add-remove-deps.md`). Read `results/T005.md` (hints section)
first: `Manifest::load`/`save` (toml_edit, comment-preserving, drops stale
deps) is the core primitive; T006 adds `add`/`remove` commands on top.
Known shape from earlier results: `run(cmd, args, cwd, verbose)` +
`build_with(manifest, root, release, verbose)` + `run_cmd`/`test_cmd`/
`clean_cmd` in `src/build.rs`; `run` propagates the child's numeric exit
code (101 only on signal death); `test` failure = carry error (exit 1) in
the tool-failure format; `clean` refuses a `build` symlink pointing outside
the project. Benign warning every configure (conan profile gnu20 vs our
CMAKE_CXX_STANDARD 23) — known, see results/T004.md.
