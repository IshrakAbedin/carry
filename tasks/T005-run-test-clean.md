# T005 — `carry run`, `carry test`, `carry clean`

Status: pending
Depends on: T004 (build pipeline)

## Goal

Complete the daily-driver commands: `carry run [args]...` (build Debug + exec),
`carry test` (build Debug + ctest), `carry clean` (remove `build/`).

## Context

- Read `DESIGN.md` §2, §6 (test/run additions) and `results/T004.md` (`run`/`build` helpers).
- Work in `src/build.rs` + `main.rs`.

## Requirements

- `fn run_cmd(root, manifest, args: &[String], verbose) -> anyhow::Result<()>`:
  `build(root, debug, verbose)` then `Command::new(root/build/bin/<name>)` with `.args(args)`,
  inherited stdio. **Propagate the child's exit code**: if non-zero, `std::process::exit(code)`
  (do not turn it into a carry `error:` — it is the program's own exit). Missing binary
  (should not happen after build) → carry error.
- `fn test_cmd(root, verbose) -> anyhow::Result<()>`: `build(debug)` then
  `ctest --test-dir build --output-on-failure` in `root`. ctest failure → carry error like
  T004's failures (summary line + non-zero exit).
- `fn clean_cmd(root)`: remove `root/build` recursively; if absent, print "nothing to clean".
  Refuse to remove if `build` exists but is a symlink pointing outside the project (safety).
- `main.rs`: wire the three commands. For `run`, remember the trailing-args clap setup from
  T001 — verify `carry run -- --release` does NOT swallow user args (carry flags must come
  before user args; document this in `--help` via the command's `about`/`long_about`).
- Unit/integration tests where cheap: `clean` on a temp dir with/without `build/` (use
  tempfile); arg-passing for `run` is covered by E2E below.

## Acceptance criteria

- [ ] `cargo test`, `cargo clippy --all-targets` green.
- [ ] Scratch bin project `runner` whose `main.cpp` prints its argc + argv (update the
      scaffold? NO — do not change the scaffold; write a one-off scratch project by hand with
      the same layout + `main.cpp` that prints args): `carry run hello world` prints the
      three args; `echo $?` → 0.
- [ ] Same project with `main.cpp` returning 3 → `carry run` exit code is 3.
- [ ] Scratch project with the default passing `test_main.cpp`: `carry test` → ctest "100% tests passed" tail in output, exit 0.
- [ ] Make a test fail (hand-edit scratch `test_main.cpp` to `assert(false)`, rebuild) → `carry test` fails with ctest output-on-failure visible, exit ≠ 0. Revert.
- [ ] `carry clean` removes `build/`; second `carry clean` prints "nothing to clean"; project still builds afterwards.

## Definition of done

- Criteria recorded in `results/T005.md` with real output tails; scratch dirs removed.
- TASKS.md/STATE.md/SESSION_LOG.md updated.
