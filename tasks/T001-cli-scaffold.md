# T001 — Rust CLI scaffold

Status: pending
Depends on: T000 (bootstrap only)

## Goal

Create the Rust project skeleton for `carry`: a clap-based CLI with all MVP subcommands
defined, error plumbing, and version output. No real logic yet — commands return a clear
"not implemented" error. This task is the foundation every later task edits.

## Context

- Read `DESIGN.md` §2 (command surface), §7 (module layout), §8 (error handling).
- The Cargo project lives at the **repo root** (`/home/ishrak/work/repo/carry/`), alongside
  the memory files. The `src/` directory already exists (empty).
- Toolchain: cargo 1.98.1. Use `cargo add` for dependencies.
- Dependencies (only these, keep it lean): `clap` (feature `derive`), `anyhow`.
  (`serde`/`toml`/`toml_edit` arrive in T002 — do not add them now.)

## Steps

1. `cargo init --name carry` at the repo root (the dir has non-code files; that is fine —
   `cargo init` does not touch them). Set `[package] version = "0.1.0"`, edition 2024 if the
   toolchain supports it (1.98 does), else 2021. Add `clap` + `anyhow` via `cargo add`.
2. Define the CLI in `src/main.rs` with clap derive:
   - Global flag `--verbose` (print each external command before running it; no behavior
     needed yet).
   - Subcommands exactly per DESIGN.md §2: `new <name> [--lib]`, `init`,
     `build [--release]`, `run [args]...` (args is `Vec<String>`, external, not a subcommand
     of its own — use `trailing_var_arg` or make `run` take `Vec<String>`), `test`,
     `clean`, `add <pkg>`, `remove <pkg>`.
   - For `run`, the simplest robust choice: `#[command(trailing_var_arg = true, allow_hyphen_values = true)] args: Vec<String>`.
3. Dispatch in `main()`: every command prints `error: <cmd> not implemented yet (planned in T0xx)` and exits 1, except `--version`/`--help` which clap handles. Reference the planned task ID in each message (build→T004, new/init→T002, run/test/clean→T005, add/remove→T006).
4. Create empty module files matching DESIGN.md §7 so later tasks only fill them in:
   `src/manifest.rs`, `src/project.rs`, `src/template.rs`, `src/generate.rs`, `src/build.rs`.
   Each gets a one-line `//!` doc comment naming the owning task. Declare them with
   `mod` in `main.rs` (unused-module warnings: add `#[allow(dead_code)]` at module level
   until T002+ fills them — remove the allow as they become used).
5. Verify acceptance criteria below.

## Acceptance criteria (run and record real output)

- [ ] `cargo build` succeeds with **zero warnings**.
- [ ] `cargo clippy --all-targets` passes with zero warnings.
- [ ] `cargo test` passes (may be 0 tests — but the harness must run).
- [ ] `cargo run -q -- --help` prints all 8 subcommands with correct flags.
- [ ] `cargo run -q -- build` prints `error: build not implemented yet (planned in T004)` and `echo $?` → 1.
- [ ] `cargo run -q -- --version` prints `carry 0.1.0`.

## Definition of done

- All acceptance criteria pass; output recorded in `results/T001.md`.
- `Cargo.toml` has only the dependencies listed above.
- TASKS.md/STATE.md/SESSION_LOG.md updated per BOOTSTRAP.md.

## Notes for the implementer

- If `cargo init` complains the dir is not empty, it only warns — proceed.
- Keep `main()` thin: one `match` on the clap enum. No helper functions yet.
