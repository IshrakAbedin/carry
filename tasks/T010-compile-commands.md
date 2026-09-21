# T010 — Compile commands export (compile_commands.json) for editor integration

Status: done (2026-09-21, user-directed ad-hoc task)
Depends on: T004 (build pipeline), T005 (clean), T008 (frozen 0.1.0 — this is a post-MVP add)

## Goal

Every `carry build` (and transitively `run`/`test`) should leave a
`compile_commands.json` where editors (clangd, VS Code C/C++) find it by
default — the **project root** — without any user configuration.

## Context

- CMake can export the database itself: `-DCMAKE_EXPORT_COMPILE_COMMANDS=ON`
  makes configure write `build/compile_commands.json`. Editors, however,
  default to looking in the source root (clangd walks up from the edited
  file; VS Code looks at the workspace root), so the `build/` copy is not
  found without manual editor config.
- carry's conventions (D3, D5): generated artifacts are written
  write-if-changed; `build/` is gitignored and wiped by `carry clean`.

## Design (→ D13)

1. Add `-DCMAKE_EXPORT_COMPILE_COMMANDS=ON` to the configure step in
   `src/build.rs`.
2. New `export_compile_commands(root)`: after configure, copy
   `build/compile_commands.json` → `<root>/compile_commands.json`,
   write-if-changed (stable mtime when the command set is unchanged, so
   editor file-watchers are not triggered on no-op rebuilds). Missing
   `build/compile_commands.json` is a hard error (cannot happen with the
   flag on — surface it loudly if it does).
3. Scaffold `.gitignore` (template.rs `GITIGNORE`) gains
   `compile_commands.json` (root copy is a build artifact; the `build/`
   copy is already covered).
4. `carry clean` removes the root `compile_commands.json` alongside
   `build/` (a stale export pointing at a wiped build/ is worse than none);
   "nothing to clean" only when neither exists.

## Steps

1. Implement 1–4 in `src/build.rs` + `src/template.rs`.
2. Unit tests (hermetic) for `export_compile_commands` (copy,
   write-if-changed/mtime, stale update, missing-source error) and for the
   clean changes (root file removed with build/; stray root file only →
   removed, no "nothing to clean").
3. Hermetic e2e: `new_scaffold_layout` asserts the scaffold `.gitignore`
   covers `compile_commands.json`.
4. `#[ignore]`d e2e `compile_commands_export` (D8): new → build → root file
   exists with a command entry per scaffold source; rebuild (no changes) →
   mtime stable; clean → root file and build/ gone.
5. Docs: DESIGN.md §2 (build/clean rows), §3 (layout), §6 (steps), decision
   D13; README (commands table, conventions, how-it-works); `Build`
   long_about in main.rs.
6. Record: results/T010.md, TASKS.md, STATE.md, SESSION_LOG.md.

## Acceptance criteria

- [x] `carry build` of a fresh project leaves a valid `compile_commands.json`
      at the project root with an entry for `src/main.cpp` and
      `tests/test_main.cpp` (and the lib sources for `--lib`).
- [x] Second `carry build` with no changes leaves the root file's mtime
      unchanged.
- [x] `carry clean` removes both `build/` and the root `compile_commands.json`.
- [x] Scaffold `.gitignore` includes `compile_commands.json`.
- [x] `cargo build` + `cargo clippy --all-targets` zero warnings;
      `cargo test` (hermetic) green; `cargo test -- --ignored` green.
- [x] DESIGN.md D13 + docs updated; recorded per BOOTSTRAP.
