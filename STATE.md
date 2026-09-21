# STATE — last updated 2026-09-21 (after T009 glaze session)

- **Current task:** none. Next: post-MVP list in DESIGN.md §10 or user direction.
- **Last completed:** T009 — glaze demo + C++ standard sync (**D12**) (2026-09-21)
- **One-line state:** carry now passes `-s compiler.cppstd=<gnuNN>` (from the manifest's
  `cpp_std`) to `conan install`, so the dependency graph is resolved with the standard the
  generated CMakeLists compiles with — a glaze/8.3.0 project (requires C++23) builds fine
  against a `cppstd=gnu20` profile; the old "modified to 23" configure warning is gone.
- **Blockers:** none
- **Open issues:** none (I001 fixed 2026-09-21)
- **Baseline checks (2026-09-21):** `cargo build` ✅ zero warnings · `cargo test` ✅ 30
  unit + 2 hermetic e2e pass (2 ignored: `fmt_full_loop`, `glaze_full_loop`) ·
  `cargo clippy --all-targets` ✅ zero warnings · `cargo test -- --ignored` ✅ both loops
  green (8.98s warm) · manual glaze demo loop (build/run/test/release) ✅
- **Environment:** cargo 1.98.1 · cmake 4.4.3 · conan 2.32.0 · g++ 16.2.1 · ninja 1.13.2
  (Linux, x86_64) · Conan Center network **verified working** · default profile
  `compiler.cppstd=gnu20` (carry overrides it per-project since D12) · fmt/10.2.1 and
  glaze/8.3.0 in local conan cache · system fmt v12 headers at `/usr/include/fmt` (why D10 matters)
- **Git:** repo-local identity `carry-dev <carry-dev@local>`. One commit per task.
  **Suggested (user decides): `git tag v0.1.0`** (MVP) — T009 is a post-freeze fix on top.
- **Deps:** clap 4.6.7 (derive), anyhow 1.0.104, serde 1.0.229 (derive), toml 1.1.6,
  toml_edit 0.25.15; dev: tempfile 3.27.0.

## New in T009 session (2026-09-21)

- **Bug:** profile `cppstd=gnu20` vs generated `CMAKE_CXX_STANDARD 23` diverged since
  T004 — benign for fmt, but glaze/8.3.0's recipe validates the standard → `conan
  install` failed ("Invalid: Current cppstd (gnu20) is lower than the required C++
  standard (23)"). Full repro + outputs in results/T009.md.
- **Fix (D12):** `conan_cppstd()` in manifest.rs (`c++23` → `gnu23`; `gnu*`/bare pass-
  throughs) + the flag in build.rs's conan install. Unit test + ignored e2e
  `glaze_full_loop` regression guard. DESIGN §4/§6 + decision D12 + README "How it works".
- **Demo-side learning (glaze 8.3.0, not carry bugs):** namespace `glz` (not `glaze`);
  `glz::write_json(v, buf)` / `glz::read_json(v, buf)` (value first); no `json_dump` in
  8.x. See results/T009.md.
- **MSVC/Clang caveat (DESIGN §10.6):** the `gnu` prefix assumes pinned GCC (D6); MSVC
  cppstd values are bare (`17`/`20`/`23`) — revisit the mapping there.

## Resume instructions

Post-MVP. A new session should: pick the top item of DESIGN.md §10 (post-MVP list)
or take user direction; follow BOOTSTRAP.md's protocol as usual. Keep the default
`cargo test` hermetic (D8): anything needing the C++ toolchain or Conan Center stays
`#[ignore]`d (currently `fmt_full_loop` + `glaze_full_loop`; run with
`cargo test -- --ignored`). `cargo test` does not rebuild the carry bin for scratch
runs — `cargo build` first. Only benign configure warning left: the `unused-cli`
CMAKE_TOOLCHAIN_FILE note on reconfigure (results/T008.md).
