# T008 — Polish: README, help text, final verification

Status: pending
Depends on: T007 (E2E proven)

## Goal

Make the MVP presentable and freeze it: real README for carry (replacing the dev-status
stub), crisp `--help` text, version stamp in generated files verified, one final clean-room
verification run.

## Context

- Read DESIGN.md in full and `results/T007.md` (the working-evidence file, incl. any
  network limitations that must be stated honestly in the README).
- This task must not change behavior — only docs, help strings, and verification.
  If you find a real bug, fix it and re-run the T007 loop; note it.

## Steps

1. **README.md** (repo root, the one that currently describes the dev workflow): restructure
   as a user-facing README:
   - What carry is (2 sentences), status (MVP, working command list, known limitations —
     from T007, e.g. offline Conan if that was the verdict).
   - Requirements (the five tools + versions tested, from BOOTSTRAP).
   - Quick start: `carry new hello && cd hello && carry build && carry run` with a sample
     of real output (paste from T007 results, trimmed).
   - Command reference table (from DESIGN §2), CARRY.toml reference (from §4).
   - "How it works" paragraph (manifest → generated CMakeLists/conanfile → conan/cmake/ninja).
   - Keep a short "AI contributors" section pointing at BOOTSTRAP.md (the memory system).
2. **Help text**: give every subcommand a one-line `about` and a `long_about` where the
   semantics need it (notably `run`: carry flags before user args). Verify with
   `carry --help`, `carry run --help`, `carry new --help`.
3. **Consistency pass**: `carry --version` matches Cargo.toml; generated-file header says
   `carry 0.1.0` (verify it reads the version from CARGO_PKG_VERSION, not hardcoded — fix
   if hardcoded).
4. **Clean-room verification** (record every command + tail in results/T008.md):
   - `cargo clean && cargo build && cargo test && cargo clippy --all-targets` — all green.
   - Fresh scratch project through the full loop once more (new --lib → add fmt if online →
     build → run → test → clean), proving the shipped state works from zero.
   - `git status` clean except intended docs changes.
5. **Freeze**: update STATE.md to "MVP complete — next: post-MVP list in DESIGN §10",
   TASKS.md all done, final SESSION_LOG entry summarizing the whole build (one paragraph).
   Suggest (do not execute) `git tag v0.1.0` in the log entry — the user decides on tags.

## Acceptance criteria

- [ ] README renders well as Markdown, contains no stale "under active development" wording
      unless still accurate.
- [ ] `cargo build` / `cargo test` / `cargo clippy --all-targets` green after `cargo clean`.
- [ ] Clean-room loop green (output in results).
- [ ] `results/T008.md` written; STATE/TASKS/SESSION_LOG updated; all nine tasks `done`.
