# T004 — Build pipeline: conan → cmake → ninja (`carry build`)

Status: pending
Depends on: T003 (generator)

## Goal

Make `carry build [--release]` actually build: the full pipeline from DESIGN.md §6
(regenerate → conan install → cmake configure → cmake --build) with robust tool invocation
and clear failure output.

## Context

- Read `DESIGN.md` §6 exactly — the commands, flags, and ordering are the spec.
- Read `results/T003.md` (regenerate API, project root helper).
- Work in `src/build.rs`.

## Requirements

- `fn run(cmd: &str, args: &[&str], cwd: &Path, verbose: bool) -> anyhow::Result<()>`:
  - If verbose, print `$ <cmd> <args...>` first.
  - `Command::new(cmd)` with inherited stdout/stderr; `.current_dir(cwd)`; spawn+wait.
  - `ErrorKind::NotFound` → `error: required tool '<cmd>' not found on PATH`.
  - Non-zero exit → anyhow error: `error: <cmd> failed with exit code <n>\n  command: $ <full line>` (do not capture output — it was streamed; only add the summary).
- `fn build(root: &Path, release: bool, verbose: bool) -> anyhow::Result<()>`:
  1. load manifest (already done by caller or here — keep the main.rs dispatch thin),
  2. `generate::regenerate` (from T003),
  3. `conan install . --output-folder=build --build=missing -s build_type=<Debug|Release>` in `root`,
  4. `cmake -S . -B build -G Ninja -DCMAKE_TOOLCHAIN_FILE=build/conan_toolchain.cmake -DCMAKE_BUILD_TYPE=<...> -DCMAKE_C_COMPILER=gcc -DCMAKE_CXX_COMPILER=g++` in `root`,
  5. `cmake --build build` in `root`.
  Step 3–5 run in `root`. Note: for a **no-dependency** project, conan still writes the
  toolchain file — keep the pipeline identical with or without deps (do not special-case).
- `main.rs`: replace the `build` stub with the real call; keep `run`/`test` stubs for now.
- `build_type` string: `Debug` (default) / `Release` (`--release`).

## Acceptance criteria (this is the first task that shells out to the real toolchain)

- [ ] `cargo test` and `cargo clippy --all-targets` still green.
- [ ] Scratch project `hello` (bin, **no deps**): `carry build` from inside it succeeds;
      `build/bin/hello` exists and running it prints the scaffold greeting. Paste the tail of the real output.
- [ ] `carry build --release` then `ls build` shows a Release build; `build/bin/hello` still works.
- [ ] Introduce a deliberate C++ syntax error in the scratch `src/main.cpp` → `carry build` fails, streams the compiler error, prints the `error: g++/ninja failed...` summary, exit code ≠ 0. Revert the error.
- [ ] `carry build --verbose` prints the `$` line for each of conan/cmake/cmake steps.
- [ ] Second `carry build` (no changes) is fast and succeeds (ninja no-op).
- [ ] **First real conan run**: if this is the first ever `conan install`, note how long
      profile/generator init takes and any warnings in results (T007 will do real deps).

## Definition of done

- All criteria recorded in `results/T004.md` with real command + tail output.
- Scratch dirs removed.
- Any deviation from DESIGN §6 recorded (and DESIGN updated if design-level).
- TASKS.md/STATE.md/SESSION_LOG.md updated.

## Notes

- conan's first `install` may be slow (it populates the cache). That is expected; time it.
- If conan warns about CMake 4.x, record the exact warning — it may matter for T007.
