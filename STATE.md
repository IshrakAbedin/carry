# STATE — last updated 2026-09-21 (after T002)

- **Current task:** T003 — Generator: manifest → conanfile.txt + CMakeLists.txt (pending, not started)
- **Last completed:** T002 — CARRY.toml manifest (serde + toml_edit save) + `carry new`/`init` scaffolds
- **One-line state:** `carry new <name> [--lib]` and `carry init` work end-to-end; build/run/test/clean/add/remove are still stubs; next is the file generator (T003).
- **Blockers:** none
- **Baseline checks (2026-09-21, current checkout):** `cargo build` ✅ zero warnings · `cargo clippy --all-targets` ✅ zero warnings · `cargo test` ✅ 9 tests pass
- **Environment:** cargo 1.98.1 · cmake 4.4.3 · conan 2.32.0 · g++ 16.2.1 · ninja 1.13.2 · Conan default profile present · Conan Center network access unverified (T007)
- **Git:** no global/system git identity on this machine; repo-local identity set to `carry-dev <carry-dev@local>` (matches T000 author). End each task with a local commit, one per task.
- **Deps so far:** clap 4.6.7 (derive), anyhow 1.0.104, serde 1.0.229 (derive), toml 1.1.6, toml_edit 0.25.15.

## Resume instructions

Next session: follow BOOTSTRAP.md's protocol and execute **T003**
(`tasks/T003-generator.md` or whatever the T003 file is named under `tasks/`).
Read `results/T002.md` (hints section) before starting: `conan_ref` and
`Manifest::load` are ready in `src/manifest.rs` (both carry
`#[allow(dead_code)]` to remove when used); `src/generate.rs` is the empty
scaffold T003 owns; placeholders to replace live in `src/template.rs`.
