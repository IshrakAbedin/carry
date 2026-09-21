# STATE — last updated 2026-09-21 (after T003)

- **Current task:** T004 — Build pipeline: conan → cmake → ninja (`carry build`) (pending, not started)
- **Last completed:** T003 — Generator: manifest → conanfile.txt + CMakeLists.txt (write-if-changed)
- **One-line state:** `new`/`init` scaffold real generated files; `build`/`run`/`test` find the project root, load the manifest, regenerate (write-if-changed), then still return "not implemented"; next is the actual build pipeline (T004).
- **Blockers:** none
- **Baseline checks (2026-09-21, current checkout):** `cargo build` ✅ zero warnings · `cargo clippy --all-targets` ✅ zero warnings · `cargo test` ✅ 14 tests pass
- **Environment:** cargo 1.98.1 · cmake 4.4.3 · conan 2.32.0 · g++ 16.2.1 · ninja 1.13.2 · Conan default profile present · Conan Center network access unverified (T007)
- **Git:** no global/system git identity on this machine; repo-local identity set to `carry-dev <carry-dev@local>` (matches T000 author). End each task with a local commit, one per task.
- **Deps so far:** clap 4.6.7 (derive), anyhow 1.0.104, serde 1.0.229 (derive), toml 1.1.6, toml_edit 0.25.15; dev: tempfile 3.27.0.

## Resume instructions

Next session: follow BOOTSTRAP.md's protocol and execute **T004**
(`tasks/T004-build-pipeline.md`). Read `results/T003.md` (hints section)
first: `prepare_project()` in `src/main.rs` already does DESIGN §6 steps 1–2
(find root → load manifest → regenerate); T004 appends steps 3–5 in
`src/build.rs` (still an empty scaffold) and removes the
"build not implemented yet" error. DESIGN §6 + D5/D6 are the spec:
`conan install . --output-folder=build --build=missing -s build_type=...`,
cmake configure with `-DCMAKE_TOOLCHAIN_FILE=build/conan_toolchain.cmake`
and pinned gcc/g++, `cmake --build build`; stream child output, error
format `error: <tool> failed with exit N`, missing-tool error
`error: required tool '<tool>' not found on PATH`. No-deps scaffold
configure/build/ctest was verified end-to-end in T003 (no Conan involved).
