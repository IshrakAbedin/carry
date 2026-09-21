# carry

**carry** is a C/C++ project manager in the spirit of Rust's **Cargo**: one small CLI and one
manifest (`CARRY.toml`) give you deterministic `build` / `run` / `test` for a C/C++ project.
Under the hood it drives **CMake** (configure), **Conan 2** (dependencies), **Ninja** (build),
and **g++** — you never touch the generated `CMakeLists.txt` or `conanfile.txt`.

> **Status: MVP (v0.1.0).** All nine commands below are implemented and verified end to end
> against a real Conan Center dependency (`fmt`) in Debug and Release. Known limitations:
> GCC-only (compilers are pinned to `gcc`/`g++`), the `default` Conan profile only, no
> lockfile, no publish/registry, no workspaces, no custom-CMake escape hatch, Linux/macOS
> paths (no Windows). See [DESIGN.md](DESIGN.md) §9–§10.

## Requirements

| Tool | Tested version | Used for |
|---|---|---|
| cargo | 1.98.1 | building carry from source (Rust edition 2024) |
| CMake | 4.4.3 | configuration (generated files require ≥ 3.21) |
| Conan | 2.32.0 | dependency resolution (`conanfile.txt`, v2 syntax) |
| g++ (and gcc) | 16.2.1 | compilation (pinned as the C/C++ compiler) |
| Ninja | 1.13.2 | build execution |

Conan needs network access to Conan Center the first time a dependency version is fetched;
subsequent builds use the local Conan cache.

## Quick start

```console
$ carry new hello
created carry project hello (type: bin)
  CARRY.toml
  .gitignore
  conanfile.txt
  CMakeLists.txt
  src/main.cpp
  tests/test_main.cpp

next: cd hello && carry build

$ cd hello
$ carry build
[1/4] Building CXX object CMakeFiles/test_hello.dir/tests/test_main.cpp.o
[2/4] Linking CXX executable bin/test_hello
[3/4] Building CXX object CMakeFiles/hello.dir/src/main.cpp.o
[4/4] Linking CXX executable bin/hello

$ carry run
ninja: no work to do.
Hello from hello!

$ carry test
100% tests passed out of 1

$ carry clean
removed build/
```

Add a dependency and use it — the generated files pick it up on the next build:

```console
$ carry add fmt@10.2.1
added fmt 10.2.1
run: carry build
```

## Commands

| Command | What it does |
|---|---|
| `carry new <name> [--lib]` | Create `./<name>/` scaffold (executable by default; `--lib` adds a library target) |
| `carry init` | Turn the current directory into a carry project (name from the directory) |
| `carry build [--release]` | Regenerate files → `conan install` → CMake configure → Ninja build (Debug default) |
| `carry run [args]...` | Build (Debug), then run `build/bin/<name>` with your args; the program's exit code is carried through |
| `carry test` | Build (Debug), then `ctest --output-on-failure` |
| `carry clean` | Remove `build/` |
| `carry add <pkg>[@<version>]` | Add or update a dependency in `CARRY.toml` (format-preserving) |
| `carry remove <pkg>` | Remove a dependency from `CARRY.toml` |
| `--verbose` (global) | Print each external command before running it |

### Dependency versions

`carry add` takes `pkg`, `pkg@version`, or `pkg@range`:

- `fmt@10.2.1` → `fmt/10.2.1` (pinned)
- `fmt@'>=10'` → `fmt/[>=10]` (range — **quote it** so the shell passes it as one argument)
- `fmt` → `fmt/[>=0]` ("any version": highest available, local cache preferred)

`conanfile.txt` and `CMakeLists.txt` are generated from `CARRY.toml` on every build and only
rewritten when their content changes, so the manifest is the single source of truth.

## CARRY.toml reference

```toml
[package]
name = "demo"          # required; [a-z0-9_][a-z0-9_-]*
version = "0.1.0"      # required; semver string
type = "bin"           # "bin" | "lib" | "both"   (default "bin")

[build]
cpp_std = "c++23"      # default "c++23"
c_std   = "c11"        # default "c11"

[dependencies]
fmt = "10.2.1"         # pkg = version or range (see above)
```

Conventions: build dir is `build/` (gitignored); executables land in `build/bin/<name>`; the
library target is `<name>_lib`; the test target `test_<name>` is registered with CTest as
`<name>_tests`; a dependency's CMake target is `<pkg>::<pkg>` (Conan 2 default).

## How it works

`carry build` reads `CARRY.toml`, renders `conanfile.txt` and `CMakeLists.txt` in memory and
rewrites them only if changed, then runs the pipeline: `conan install . --output-folder=build
--build=missing -s build_type=<Debug|Release>` (writes the Conan toolchain + CMakeDeps
configs into `build/`), `cmake -S . -B build -G Ninja` with the Conan toolchain file, and
`cmake --build build`. Build output streams straight to the terminal; a tool failure reports
the tool, its exit code, and the exact command line.

## For AI contributors

This repository uses a **repo-as-memory** workflow: project state, the current task, task
prompts, and past results live in files because AI session context is cleared between runs.
**Start every session by reading [`BOOTSTRAP.md`](BOOTSTRAP.md)** and following its protocol;
paste-ready prompts are in [`prompts/`](prompts/).

| File | Purpose |
|---|---|
| `BOOTSTRAP.md` | Session protocol — read first |
| `STATE.md` | Current state + next task (kept small) |
| `DESIGN.md` | Architecture & decisions (stable reference) |
| `TASKS.md` | Backlog & status |
| `tasks/` | Self-contained per-task prompts |
| `results/` | Per-task outcomes (decisions, verification) |
| `ISSUES.md` | Open issues backlog |
| `SESSION_LOG.md` | Append-only session history (newest on top) |

## Building carry

```console
$ git clone <this repo> && cd carry
$ cargo build
```

The binary lands in `target/debug/carry` (or `target/release/carry` with `cargo build
--release`). Tests: `cargo test` (hermetic by default); the E2E loop against a real Conan
dependency is `#[ignore]`d and runs via `cargo test -- --ignored`.
