# T007 — E2E hardening with a real Conan dependency (fmt)

Status: pending
Depends on: T006 (full command set exists)

## Goal

Prove the whole loop works against **real Conan Center packages**, exercise the
`#[ignore]`d E2E tests, and fix every bug this surfaces. This is the "does it actually
work" task — expect iteration; that is the point.

## Context

- Read DESIGN.md §6 and the `results/` of T004–T006 (known quirks, conan warnings noted there).
- This task is where the **network access question** from BOOTSTRAP gets answered.

## Steps

1. **Network probe**: in a scratch dir, run
   `conan install . --output-folder=build --build=missing -s build_type=Debug` with a minimal
   `conanfile.txt` requiring `fmt/[>=10]`. Record: works / fails (with exact error).
   - If offline: record the verdict in `results/T007.md` + `ISSUES.md` (I: "Conan Center
     unreachable"), fall back to the hermetic path below, and mark this task done-with-fallback
     (T008 must then state the limitation in the README).
2. **Full loop (online path)** — scratch project `fmtapp` (bin + lib via `carry new fmtapp --lib`):
   - `carry add fmt@10.2.1`
   - Edit `src/main.cpp` (and `lib.cpp`/hpp) to actually **use** `fmt` (e.g.
     `fmt::print("answer: {}\n", 42);`) — this is a scratch project, hand-editing is fine.
   - `carry build` (Debug) → succeeds; `carry run` prints `answer: 42`.
   - `carry build --release` → succeeds.
   - `carry test` → passes (test binary links fmt via generated CMake).
   - `carry remove fmt`, revert main.cpp, `carry build` → still green (dep removal path).
3. **Un-ignore the E2E tests**: convert the manual loop from step 2 into
   `tests/e2e.rs` tests marked `#[ignore]` (they spawn the real `carry` binary via
   `env!("CARGO_BIN_EXE_carry")` or a `CARRY_BIN` override; use `tempfile` dirs).
   Run `cargo test -- --ignored` and record real durations (conan cache is warm now, so
   they should be minutes, not hours).
4. **Bug pass**: fix any crash, bad error message, missing edge case found in 1–3
   (e.g. conan warnings about CMake 4.x, missing include dirs for lib targets, ctest not
   finding the test, `run` arg edge cases). Each fix gets a line in results + a regression
   test where feasible. Re-run the acceptance criteria after fixes.

## Acceptance criteria

- [ ] Network verdict recorded (online: full loop green; offline: documented fallback).
- [ ] `fmtapp` loop output (tail of each command) in `results/T007.md`.
- [ ] `cargo test -- --ignored` green (online path) with durations recorded.
- [ ] `cargo test` (hermetic suite) + `cargo clippy --all-targets` green.
- [ ] All bugs found fixed or filed in ISSUES.md with status.
- [ ] Scratch dirs removed.

## Definition of done

- `results/T007.md` is the "it really works" evidence file: verdict, loop output, fixes list.
- STATE.md/SESSION_LOG.md updated; if the MVP is now demonstrably working end-to-end, say so
  explicitly in STATE's one-line state (T008 then focuses on polish, not correctness).
