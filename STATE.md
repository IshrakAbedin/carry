# STATE — last updated 2026-09-18 (after T001)

- **Current task:** T002 — Manifest (CARRY.toml) + `carry new` / `carry init` (pending, not started)
- **Last completed:** T001 — Rust CLI scaffold: clap CLI with all 8 subcommands, stubs, error plumbing
- **One-line state:** Crate builds clean at repo root; every subcommand is a stub printing "not implemented (planned in T0xx)"; next is the manifest + scaffolding.
- **Blockers:** none
- **Baseline checks (2026-09-18, current checkout):** `cargo build` ✅ zero warnings · `cargo clippy --all-targets` ✅ zero warnings · `cargo test` ✅ 0 tests, harness runs
- **Environment:** cargo 1.98.1 · cmake 4.4.3 · conan 2.32.0 · g++ 16.2.1 · ninja 1.13.2 · Conan default profile present · Conan Center network access unverified (T007)
- **Git:** no global/system git identity on this machine; repo-local identity set to `carry-dev <carry-dev@local>` (matches T000 author). End each task with a local commit, one per task.
- **Deps so far:** clap 4.6.7 (derive), anyhow 1.0.104. T002 adds serde/toml/toml_edit via `cargo add`.

## Resume instructions

Next session: follow BOOTSTRAP.md's protocol and execute **T002** (`tasks/T002-manifest-new-init.md` or whatever the T002 file is named under `tasks/`).
Read `results/T001.md` (hints section) before starting: module-allow locations, stderr error
convention, and the stub-dispatch seam in `src/main.rs` are all documented there.
