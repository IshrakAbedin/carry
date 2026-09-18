# SESSION LOG — newest on top

Append one entry per session: date, task(s) worked, what was done, verification result,
decisions (pointer to DESIGN.md if any), anything the next session must know.
Keep each entry to ~10 lines max; deep detail belongs in `results/`.

---

## 2026-09-18 — T001: Rust CLI scaffold

- `cargo init` (edition 2024) at repo root; deps `clap 4.6.7` (derive) + `anyhow 1.0.104` only.
- `src/main.rs`: all 8 MVP subcommands + global `--verbose`; each stub prints
  `error: <cmd> not implemented yet (planned in T0xx)` to stderr, exit 1.
- Empty module scaffolds per DESIGN §7 (manifest/template→T002, generate→T003,
  project/build→T004/T005); `#[allow(dead_code)]` sits on the `mod` lines in main.rs
  (a bare allow in an empty module file is a syntax error) — remove per module as used.
- Task-note corrections found: run's trailing args need `#[arg(...)]`, not `#[command(...)]`.
- Verification: build/clippy zero warnings, test harness runs (0 tests), `--help` shows all
  8 subcommands, `build` stub exits 1, `--version` → `carry 0.1.0`. See results/T001.md.
- Next: T002 (manifest + new/init). Conan Center network still unverified (T007).

---

## 2026-09-17 — T000: bootstrap

- Verified environment: cargo 1.98.1, cmake 4.4.3, conan 2.32.0 (v2, default profile present),
  g++ 16.2.1, ninja 1.13.2. Repo was empty; initialized git (branch `main`).
- Wrote the repo-as-memory system: BOOTSTRAP.md (session protocol), STATE.md, TASKS.md,
  ISSUES.md, this log, prompts/ (next-task, status, fix-issue).
- Wrote DESIGN.md: full carry design (commands, CARRY.toml schema, generated files, build
  pipeline, Rust module layout, D1–D8 decisions).
- Broke the MVP into tasks T001–T008 under tasks/ with self-contained prompts.
- Verification: `git log` shows initial commit; directory layout matches DESIGN.md §7.
- Next: T001 (Rust CLI scaffold). Note: Conan Center network access still unverified (T007).
