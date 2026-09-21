# STATE — last updated 2026-09-21 (after T004)

- **Current task:** T005 — `carry run`, `carry test` (ctest), `carry clean` (pending, not started)
- **Last completed:** T004 — Build pipeline: conan → cmake → ninja (`carry build`)
- **One-line state:** `carry build [--release]` runs the full real pipeline (regenerate → conan install → cmake configure → cmake --build) with streamed output and proper error/verbose handling; `run`/`test`/`clean` are still not-implemented stubs; next is T005.
- **Blockers:** none
- **Open issues:** I001 — conan writes `CMakeUserPresets.json` into the project root, which the scaffold `.gitignore` doesn't cover (fix belongs to template.rs; see ISSUES.md).
- **Baseline checks (2026-09-21, current checkout):** `cargo build` ✅ zero warnings · `cargo clippy --all-targets` ✅ zero warnings · `cargo test` ✅ 16 tests pass
- **Environment:** cargo 1.98.1 · cmake 4.4.3 · conan 2.32.0 · g++ 16.2.1 · ninja 1.13.2 · Conan default profile present · Conan Center network access still unverified (T007); first real `conan install` (no deps) ran fine in ~0.5 s — cache/profile pre-initialized, no CMake 4.x warnings.
- **Git:** no global/system git identity on this machine; repo-local identity set to `carry-dev <carry-dev@local>` (matches T000 author). End each task with a local commit, one per task.
- **Deps so far:** clap 4.6.7 (derive), anyhow 1.0.104, serde 1.0.229 (derive), toml 1.1.6, toml_edit 0.25.15; dev: tempfile 3.27.0.

## Resume instructions

Next session: follow BOOTSTRAP.md's protocol and execute **T005**
(`tasks/T005-run-test-clean.md`). Read `results/T004.md` (hints section)
first: `src/build.rs` now has the `run(cmd, args, cwd, verbose)` helper
(streamed output, missing-tool + non-zero-exit error formats per DESIGN §6)
and `build(root, release, verbose)` (full §6 pipeline, loads manifest
itself). T005: add `test()` (build Debug + `ctest --test-dir build
--output-on-failure`), `run_app()` (build Debug + exec `build/bin/<name>`,
**propagate the child exit code** — `run()` errors on non-zero, so it needs a
status-returning variant or special-case; DESIGN §8: exit 101 convention),
and `clean()` (remove `build/`); wire the three stubs in main.rs and delete
`prepare_project()` (its work is inside `build()` now). Benign warning every
configure (conan profile gnu20 vs our CMAKE_CXX_STANDARD 23) — known, see
results/T004.md.
