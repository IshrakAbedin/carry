# T009 — Demo build with glaze; sync the C++ standard between conan and CMake

Status: done (2026-09-21, user-directed ad-hoc task)
Depends on: T007 (E2E with a real dep), T008 (frozen 0.1.0)

## Goal

Run a temporary demo project through carry using the **glaze** library from
Conan Center, and fix the C++ version issues that surface.

## Context

- glaze (stephenberry/glaze) is a header-only C++20/23 serialization library;
  the Conan Center recipe for 8.3.0 **validates `compiler.cppstd`** and
  requires C++23.
- The machine's default Conan profile is `compiler.cppstd=gnu20`, while the
  generated `CMakeLists.txt` pins `CMAKE_CXX_STANDARD 23`. The two had
  diverged since T004: benign for fmt (never checks), fatal for glaze.

## Steps

1. `carry new glazedemo` in a /tmp scratch dir; `carry add glaze` (bare → D11
   `[>=0]` range); write a small `main.cpp` + test that serialize/parse a
   struct with glaze.
2. `carry build` — reproduce the failure; record the exact conan error.
3. Fix in carry (not in the generated files — BOOTSTRAP hard rule): pass the
   manifest's `cpp_std` to `conan install` as `-s compiler.cppstd=<gnuNN>`.
4. Re-run the full demo loop (build/run/test/release) and the ignored E2E
   loops; add an ignored `glaze_full_loop` e2e as a regression guard.
5. Record: results/T009.md, DESIGN.md decision (D12), §4/§6 updates,
   README "How it works", TASKS/STATE/SESSION_LOG.

## Acceptance criteria

- [ ] `carry build` of a glaze project succeeds with profile `cppstd=gnu20`
      and manifest `cpp_std = "c++23"`.
- [ ] `carry run` prints the serialized JSON + a parse round-trip.
- [ ] `cargo test` (hermetic) + `cargo clippy --all-targets` green;
      `cargo test -- --ignored` green (fmt + glaze loops).
- [ ] DESIGN.md decision + docs updated; recorded per BOOTSTRAP.
