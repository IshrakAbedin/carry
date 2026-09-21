# STATE — last updated 2026-09-21 (after T010 compile-commands session)

- **Current task:** none. Next: post-MVP list in DESIGN.md §10 or user direction.
- **Last completed:** T010 — compile commands export (**D13**) (2026-09-21)
- **One-line state:** every `carry build` now exports the compile database to
  the project **root** (`compile_commands.json`, write-if-changed) so
  clangd/VS Code work with zero configuration; `carry clean` removes it
  alongside `build/`; scaffold `.gitignore` covers it.
- **Blockers:** none
- **Open issues:** none (I001 fixed 2026-09-21)
- **Baseline checks (2026-09-21):** `cargo build` ✅ zero warnings ·
  `cargo test` ✅ 36 unit + 2 hermetic e2e pass (3 ignored:
  `fmt_full_loop`, `glaze_full_loop`, `compile_commands_export`) ·
  `cargo clippy --all-targets` ✅ zero warnings · `cargo test -- --ignored`
  ✅ all three loops green (9.61s warm) · manual scratch loop
  (build → root export w/ 3 entries + `-std=gnu++23` → no-op rebuild mtime
  stable → clean removes both) ✅
- **Environment:** cargo 1.98.1 · cmake 4.4.3 · conan 2.32.0 · g++ 16.2.1 · ninja 1.13.2
  (Linux, x86_64) · Conan Center network **verified working** · default profile
  `compiler.cppstd=gnu20` (carry overrides it per-project since D12) · fmt/10.2.1 and
  glaze/8.3.0 in local conan cache · system fmt v12 headers at `/usr/include/fmt` (why D10 matters)
- **Git:** repo-local identity `carry-dev <carry-dev@local>`. One commit per task.
  **Suggested (user decides): `git tag v0.1.0`** (MVP) — T009/T010 are post-freeze adds on top.
- **Deps:** clap 4.6.7 (derive), anyhow 1.0.104, serde 1.0.229 (derive), toml 1.1.6,
  toml_edit 0.25.15; dev: tempfile 3.27.0.

## New in T010 session (2026-09-21)

- **D13 (DESIGN.md §2/§3/§6 + decision):** configure gets
  `-DCMAKE_EXPORT_COMPILE_COMMANDS=ON`; `export_compile_commands()` in
  build.rs copies `build/compile_commands.json` → project root
  write-if-changed (stable mtime on no-op rebuilds — editor
  file-watchers not triggered). Copy (not symlink), placed between
  configure and build so a failing ninja still leaves a current command
  list. No new subcommand — side effect of the pipeline.
- **`carry clean`** now removes the root `compile_commands.json` too
  ("nothing to clean" only when neither artifact exists; stray-file-only
  case handled); scaffold `.gitignore` += `compile_commands.json`.
- **CMake/Ninja export detail:** `"file"` entries are **absolute** paths —
  the ignored e2e `compile_commands_export` anchors assertions on
  `<root>/<rel>`. Clang would emit `"arguments"` lists instead of
  `"command"` (results/T010.md hint; nothing in carry parses the file).

## Resume instructions

Post-MVP. A new session should: pick the top item of DESIGN.md §10 (post-MVP list)
or take user direction; follow BOOTSTRAP.md's protocol as usual. Keep the default
`cargo test` hermetic (D8): anything needing the C++ toolchain or Conan Center stays
`#[ignore]`d (currently `fmt_full_loop` + `glaze_full_loop` +
`compile_commands_export`; run with `cargo test -- --ignored`). `cargo test` does not
rebuild the carry bin for scratch runs — `cargo build` first. Only benign configure
warning left: the `unused-cli` CMAKE_TOOLCHAIN_FILE note on reconfigure (results/T008.md).
