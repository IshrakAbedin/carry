# SESSION LOG — newest on top

Append one entry per session: date, task(s) worked, what was done, verification result,
decisions (pointer to DESIGN.md if any), anything the next session must know.
Keep each entry to ~10 lines max; deep detail belongs in `results/`.

---

## 2026-09-21 — T010: compile commands export for editors (D13)

- **User-directed ad-hoc task (post-MVP):** make editors (clangd, VS Code) work on
  carry projects with zero config — generate a `compile_commands.json`.
- **→ D13** (DESIGN.md §2/§3/§6 + decision): configure gains
  `-DCMAKE_EXPORT_COMPILE_COMMANDS=ON`; new `export_compile_commands()` in build.rs
  copies `build/compile_commands.json` to the project **root** (where editors look by
  default), write-if-changed so no-op rebuilds keep the mtime stable; placed before
  `cmake --build` so a failing ninja still leaves a current command list. Copy, not
  symlink; no new subcommand.
- **`carry clean`** now removes the root export alongside `build/` (stale export >
  none is worse than none); scaffold `.gitignore` += `compile_commands.json`; `Build`
  help text updated.
- Verification: hermetic `cargo test` 36 unit + 2 e2e green, clippy zero warnings;
  `cargo test -- --ignored` → **compile_commands_export (new) + fmt + glaze all ok
  (9.61s)**; scratch loop confirmed absolute `"file"` paths, `-std=gnu++23`, mtime
  stability, clean output. One commit (T010).

## 2026-09-21 — T009: glaze demo + C++ standard sync (D12)

- **User-directed ad-hoc task (post-MVP):** run a temporary demo project through carry
  using the **glaze** library and fix the C++ version issues. Demo
  (`/tmp/glazedemo`: new → bare `add glaze` → JSON serialize/round-trip main + test).
- **Bug:** profile `cppstd=gnu20` vs generated `CMAKE_CXX_STANDARD 23` diverged since
  T004 — benign for fmt, but glaze/8.3.0's recipe *validates* the standard and
  `conan install` failed: "Invalid: Current cppstd (gnu20) is lower than the required
  C++ standard (23)" (conan exit 6), before any compilation.
- **Fix → D12** (DESIGN.md §4/§6 + decision): `conan install` now gets
  `-s compiler.cppstd=<gnuNN>` derived from the manifest `cpp_std` (`c++23` → `gnu23`);
  `conan_cppstd()` in manifest.rs + flag in build.rs. Profile still governs everything
  else. Bonus: the old "modified to 23" configure warning is gone.
- **Demo-side (glaze 8.3.0 API, not carry bugs):** namespace is `glz`; entry points are
  `glz::write_json(v, buf)` / `glz::read_json(v, buf)` (value first) — no `json_dump`
  in 8.x.
- Verification: hermetic `cargo test` 30 unit + 2 e2e green, clippy zero warnings;
  `cargo test -- --ignored` → **glaze_full_loop (new regression guard) + fmt_full_loop
  both ok (8.98s)**; manual demo loop build/run/test/release all green
  (`serialized: {"name":"Alice","age":42}` + round-trip). One commit (T009).

## 2026-09-21 — T008: Polish — MVP frozen (closing entry for the MVP build)

- **T008 done; all nine tasks (T000–T008) complete — carry 0.1.0 is frozen.** No behavior
  changes needed: consistency pass found the version stamp already `CARGO_PKG_VERSION`-driven
  and `carry --version` matching Cargo.toml. Work = README rewritten (user-facing: status +
  honest limitations, requirements table, quick start with real captured output, command +
  CARRY.toml references, version-spec incl. D11 quoting, "how it works", AI-contributors
  pointer), and help polish in `src/main.rs` (top-level/`build`/`run`/`add` `long_about`s,
  `run [ARGS]` help, exit-code propagation note).
- Verification (clean room): `cargo clean && cargo build` (5.29s, zero warnings) →
  `cargo test` (29 unit + 2 hermetic e2e ok, `fmt_full_loop` ignored per D8) →
  `cargo clippy --all-targets` (zero warnings); two fresh scratch loops from zero — plain
  `hello` (new→build→run) and `fmtapp --lib` + `fmt@10.2.1` (new→add→build→run→test→clean,
  `100% tests passed`) — all green; `git status` clean except README + main.rs.
- One benign note recorded, not filed: re-configure prints CMake `unused-cli` warning for
  `CMAKE_TOOLCHAIN_FILE` (cached; reconfigure-every-build is intentional, DESIGN §6).
- **The whole MVP build in one breath (2026-09-17 → 2026-09-21, 9 tasks + 1 issue fix):**
  bootstrap of the repo-as-memory system (T000), clap CLI skeleton with error plumbing
  (T001), CARRY.toml manifest + `new`/`init` (T002), the manifest→conanfile/CMakeLists
  generator (T003), the conan→cmake→ninja build pipeline (T004), `run`/`test`/`clean`
  (T005), format-preserving `add`/`remove` (T006), E2E hardening with a real fmt
  dependency that surfaced and fixed 3 real bugs (T007, decisions D9–D11), the I001
  .gitignore fix, and this polish pass (T008). Next: post-MVP list in DESIGN.md §10.
- **Suggested (user decides, not executed): `git tag v0.1.0` on the T008 commit.**

## 2026-09-21 — Fix-issue session: I001 (CMakeUserPresets.json un-ignored)

- **I001 fixed at root cause** (user-directed: fix open issues only, T008 polish deferred).
  Reproduced exactly as filed (scratch `carry new demo` + git init/commit →
  `carry build` → `git status` shows `?? CMakeUserPresets.json` at root; no
  other strays exist — no `.bak`-style files in carry's code). Fix: `GITIGNORE`
  in `src/template.rs` is now `build/\nCMakeUserPresets.json\n` (scaffold-domain
  fix per the issue's own notes; conan's root-write behavior is untouched).
  One-line comment update in DESIGN.md §3 layout (recorded here per the
  no-silent-DESIGN-rules).
- Regression tests: hermetic `new_scaffold_layout` now asserts the scaffold
  `.gitignore` contains `build/` + `CMakeUserPresets.json`; `#[ignore]`d
  `fmt_full_loop` asserts after its real build that the stray exists at root
  **and** is covered by the scaffold `.gitignore`.
- Verification: repro re-run post-fix — same commands, `git status` after
  build is clean. `cargo build`/`clippy --all-targets` zero warnings,
  `cargo test` 29 unit + 2 e2e pass, `--ignored` `fmt_full_loop` **5.01s**
  (warm). Scratch removed. No new issues. Next: T008 (polish only — I001 no
  longer in scope).

## 2026-09-21 — T007: E2E hardening with a real Conan dependency (fmt)

- **MVP proven E2E:** fresh scratch `fmtapp --lib` using fmt in lib+bin+test:
  add fmt@10.2.1 → build Debug → run prints `answer: 42` → build --release
  (CMakeCache Release) → test `100% tests passed` → remove fmt + revert →
  build green, conanfile without fmt. Network probe: works (range resolution
  0.26s warm; conancenter lists fmt ≤ 12.2.0).
- **3 bugs found, all fixed (D10, D11 in DESIGN.md):** B1 generated lib had no
  dep includes/links → system-fmt header hijack, link failure on
  `fmt::v12`/`v10` inline-namespace mismatch → lib now links deps PUBLIC;
  B2 `add fmt@` (trailing @) silently meant "any" → now error + `help:`;
  B3 bare `pkg` → `pkg/*` is **not resolvable by Conan 2.32 as a require**
  (even with the version cached) → now `pkg/[>=0]`.
- **tests/e2e.rs** (new): hermetic `new_scaffold_layout` +
  `add_remove_round_trip` (incl. trailing-@ guard) in the default suite;
  `fmt_full_loop` `#[ignore]`d (D8) = the whole manual loop as code.
- Verification: 29 unit + 2 e2e pass, clippy clean, `--ignored` loop **5.38s**
  (warm). Scratch dirs removed. No new issues; I001 re-reproduced, stays open.
  Next: T008 (polish: README/help, I001 `.gitignore` fix, final verification).

## 2026-09-21 — T006: carry add / carry remove (format-preserving)

- `manifest.rs`: `parse_dep_spec` (split on **last** `@`, name rule via new
  labeled validator so errors say "invalid dependency name"),
  `Manifest::add_dep` (same version → no write at all, mtime preserved; replace
  → in-place line update; new → table appended with blank line; prints
  `added`/`updated`/`already up to date` + `run: carry build` hint after a
  write), `Manifest::remove_dep` (absent → exact error, empty `[dependencies]`
  left in place, tolerates `pkg@version`). No manual toml_edit decoration code
  needed — probes confirmed toml_edit 0.25 handles new-table/new-key
  serialization. `main.rs`: arms wired, stub closure deleted, `add` help text
  fixed (`fmt@10.2.1`, not `fmt/10.2.1`). +8 unit tests (28 total) incl.
  exact-raw-string formatting-preservation test.
- Verification: build/clippy zero warnings, **28 unit tests pass**; scratch
  with hand-written commented CARRY.toml: add → diff = only the new line,
  `# my deps` intact; re-add same → no file change; add 11.0.0 → in-place
  update; remove → key gone, header + comment remain; remove again →
  `error: dependency 'fmt' not found in CARRY.toml` exit 1; `add Bad-Pkg` →
  error + `help:` exit 1. See results/T006.md.
- **Conan Center network verified working** (resolves the T007 caveat):
  `carry build` with `fmt/10.2.1` downloaded the recipe from conancenter and
  built the binary via `--build=missing`; full pipeline exit 0. No new issues
  (I001 unchanged). Next: T007 — use fmt *in code* end to end.

## 2026-09-21 — T005: carry run / test (ctest) / clean

- `build.rs`: `build()` split into public wrapper + private `build_with`
  (shared manifest load); added `run_cmd` (build Debug → exec
  `build/bin/<name>`, **child exit code propagated** via
  `process::exit(code)`, 101 only on signal death, missing binary → carry
  error), `test_cmd` (build Debug → `ctest --test-dir build
  --output-on-failure`, failure = carry error exit 1 in T004 format),
  `clean_cmd` (remove `build/`; "nothing to clean" if absent; refuses a
  `build` symlink pointing outside the project, inside/broken links removed
  link-only). `main.rs`: three arms wired, `prepare_project()` deleted,
  `run` `long_about` documents the `--` separator (carry flags before user
  args). +4 unit tests (clean, tempfile; symlink cases).
- Verification: build/clippy zero warnings, **20 unit tests pass**; scratch
  `runner` (hand-written layout, scaffold untouched): `run hello world` →
  argc 3 + both args, exit 0; `run -- --release` → `--release` reached the
  child (not swallowed); `main` returns 3 → carry exit **3**; `test` →
  "100% tests passed", exit 0; `assert(false)` → on-failure output visible,
  `error: ctest failed with exit code 8`, exit 1 (reverted); `clean` →
  `removed build/`, 2nd → `nothing to clean`, rebuild OK. See results/T005.md.
- No new issues (I001 unchanged). Next: T006 (add/remove on
  `Manifest::load`/`save`); hints in results/T005.md.

---

## 2026-09-21 — T004: Build pipeline (conan → cmake → ninja)

- `build.rs`: `run()` (streamed stdout/stderr, verbose `$` line, missing-tool
  + non-zero-exit errors per DESIGN §6) and `build()` (load → regenerate →
  conan install → cmake configure → cmake --build, identical with/without
  deps). `main.rs`: Build arm wired; run/test stubs kept for T005.
- **Bug caught by acceptance:** conan's `-s build_type=…` must be two argv
  tokens — one combined token made conan parse the setting name as
  `settings. build_type` (with a space) and fail. Fixed; CMake `-D` flags are
  correctly single tokens.
- Verification: build/clippy zero warnings, **16 unit tests pass** (+2 for
  `run()` error messages); scratch `hello` (no deps): first real conan install
  OK in ~1.1 s (cache pre-initialized, no CMake 4.x warning), binary prints
  greeting, `--release` flips CMakeCache to Release, deliberate syntax error
  streams g++ diagnostics + `error: cmake failed with exit code 1` (exit 1),
  `--verbose` prints all three `$` lines, no-op rebuild 0.3 s
  (`ninja: no work to do`). See results/T004.md.
- **New issue I001:** conan writes `CMakeUserPresets.json` at project root,
  un-covered by scaffold `.gitignore` (template-domain fix, deferred).
- Next: T005 (run/test/clean). Conan Center network still unverified (T007).

---

## 2026-09-21 — T003: Generator (manifest → conanfile.txt + CMakeLists.txt)

- `generate.rs`: `render_conanfile`/`render_cmake` (DESIGN §5) + `regenerate`
  (write-if-changed, returns changed-flags); `project.rs`: `find_project_root`
  (walk up for CARRY.toml); `main.rs`: build/run/test call `prepare_project()`
  (root → load → regenerate) before their not-implemented errors; `new`/`init`
  now scaffold the real rendered files (placeholders deleted).
- **Deviation → new decision D9 (dated in DESIGN.md + §5 line fixed):** the
  §5 example's `target_link_libraries(test_demo PRIVATE demo ...)` does not
  configure — CMake 4 forbids linking an EXECUTABLE into another target (and
  `ENABLE_EXPORTS` would duplicate `main`). Test target now links the library
  (if present) + deps only. Found by CMake-configuring the scratch scaffold.
- Verification: build/clippy zero warnings, **14 unit tests pass** (3 goldens
  + idempotence mtime-asserted + stale-rewrite); scratch: bin demo
  conanfile = `[generators]` only, CMake configures + builds + ctest 100%
  (no deps); hand-added fmt/zlib → 1st build regenerates (find_package +
  `::` links), 2nd build mtimes unchanged; subdir discovery OK; outside dir
  → `error: not a carry project (no CARRY.toml found)`; `--lib` scaffold
  renders both targets, configures OK. See results/T003.md.
- Next: T004 (build pipeline: conan → cmake → ninja). DESIGN §6 + D5/D6 are
  the spec. Conan Center network still unverified (T007).

---

## 2026-09-21 — T002: Manifest (CARRY.toml) + carry new/init

- Found the working tree with an **interrupted previous session's uncommitted
  T002 work** (manifest.rs/template.rs implemented, STATE.md still "pending").
  Reviewed the diff against T002 scope, verified it, and completed the task.
- `manifest.rs`: serde schema w/ defaults, `load`/`save` (toml_edit,
  comment-preserving, drops stale deps), validation w/ `help:` hints, `conan_ref`.
- `template.rs`: scaffold consts + `cmd_new`/`cmd_init` (`{ident}` = dashes→
  underscores for C++ names); placeholders for conanfile.txt/CMakeLists.txt (T003).
- `main.rs`: stub dispatch seam became `run(cli) -> anyhow::Result` (old `-> !`
  closure can't carry real errors); New/Init wired, six stubs unchanged.
- Verification: build/clippy zero warnings, **9 unit tests pass**, scratch-dir
  checks all green (new --lib tree = DESIGN §3, g++ compile+run OK, all error
  paths exit 1 with help hints). See results/T002.md.
- Next: T003 (generator: manifest → conanfile.txt + CMakeLists.txt).
  Conan Center network still unverified (T007).

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
