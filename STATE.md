# STATE — last updated 2026-09-21 (after I001 fix session)

- **Current task:** T008 — Polish: README, help text, final verification (pending, not started)
- **Last completed:** I001 fixed (dedicated fix-issue session, 2026-09-21)
- **One-line state:** MVP demonstrably works E2E — a real Conan Center dep (fmt) is add/resolve/build/run/test/remove'd through carry-generated files (Debug + Release), the one known wart (I001) is fixed, so T008 is pure polish.
- **Blockers:** none
- **Open issues:** none (I001 fixed 2026-09-21: scaffold `.gitignore` now covers `CMakeUserPresets.json`)
- **Baseline checks (2026-09-21, current checkout):** `cargo build` ✅ zero warnings · `cargo clippy --all-targets` ✅ zero warnings · `cargo test` ✅ 29 unit + 2 hermetic e2e pass · `cargo test -- --ignored` ✅ `fmt_full_loop` 5.01s (warm cache)
- **Environment:** cargo 1.98.1 · cmake 4.4.3 · conan 2.32.0 · g++ 16.2.1 · ninja 1.13.2 (Linux, x86_64) · Conan Center network **verified working** (T006/T007) · fmt/10.2.1 in local conan cache (Debug + Release) · **system fmt v12 headers at `/usr/include/fmt` on this machine** (what made B1/D10 visible; a generated lib must link deps to avoid the hijack)
- **Git:** repo-local identity `carry-dev <carry-dev@local>` (no global identity on this machine). End each task with a local commit, one per task.
- **Deps so far:** clap 4.6.7 (derive), anyhow 1.0.104, serde 1.0.229 (derive), toml 1.1.6, toml_edit 0.25.15; dev: tempfile 3.27.0.

## New in I001 fix session (2026-09-21)

- **I001 fixed:** `GITIGNORE` in `src/template.rs` is now `build/\nCMakeUserPresets.json\n` — conan's CMakeToolchain writes that file into the project root on every `conan install`; scaffold `.gitignore` now covers it so git-managed projects stay clean. Conan's root-write behavior itself is unchanged (the .gitignore fix was the issue's own recommended scope).
- Regression tests: hermetic `new_scaffold_layout` asserts scaffold `.gitignore` contains both entries; `#[ignore]`d `fmt_full_loop` asserts after a real build that the stray exists at root **and** is covered. Verified by re-running the exact I001 repro: `git status` after `carry build` is now clean.

## Prior decisions (read results/T007.md, DESIGN.md §Decisions for detail)

- **D10:** generated **lib** target links deps **PUBLIC** — without it, `#include <fmt/format.h>` in `src/lib.cpp` resolves to system fmt headers while linking Conan's lib → link failure on inline-namespace mismatch (`fmt::v12` vs `fmt::v10`). Note: **system fmt v12 headers exist at `/usr/include/fmt` on this machine** (this is what made B1/D10 visible; a generated lib must link deps to avoid the hijack).
- **D11:** bare `pkg` (no version) maps to `pkg/[>=0]`, **not** `pkg/*` — Conan 2.32 rejects a bare `*` as a require (unresolvable even with the version cached). `carry add fmt@` (trailing `@`) is rejected with a `help:` hint.
- `tests/e2e.rs`: 2 hermetic default tests + `#[ignore]`d `fmt_full_loop` (run via `cargo test -- --ignored`); `CARRY_BIN` env override supported.

## Resume instructions

Next session: follow BOOTSTRAP.md's protocol and execute **T008**
(`tasks/T008-*.md`, check TASKS.md). I001 is **done** — T008 is polish only:
write the real README (commands, the `pkg@version` spec — ranges need shell
quoting: `carry add fmt@'>=10'` — bare `pkg` = any version, the `run: carry
build` hint), polish clap help text, and do the final verification pass.
Keep the default `cargo test` hermetic (D8): anything needing the C++
toolchain or Conan Center stays `#[ignore]`d. Watch out: `cargo test` does
not rebuild the carry bin for scratch runs — `cargo build` first. Known
benign warning every configure: conan profile gnu20 vs CMAKE_CXX_STANDARD 23
(results/T004.md).
