# carry

A C/C++ project manager in the spirit of Rust's **Cargo**: one small CLI, one manifest
(`CARRY.toml`), deterministic `build` / `run` / `test`. Under the hood it drives
**CMake** (configure), **Conan 2** (dependencies), **Ninja** (build), and **g++**.

> **Status: under active development.** See `TASKS.md` for progress and `DESIGN.md` for the
> full design.

## For AI coding sessions

This repository uses a **repo-as-memory** workflow: the project state, current task, task
prompts, and past results are cached in files, because session context is cleared between
runs. **Start every session by reading [`BOOTSTRAP.md`](BOOTSTRAP.md)** and following its
protocol. Quick prompts live in [`prompts/`](prompts/).

Key files:

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

## Planned CLI (MVP)

```
carry new <name> [--lib]      scaffold a project
carry init                    scaffold in the current directory
carry build [--release]       conan → cmake → ninja
carry run [args]...           build + run
carry test                    build + ctest
carry clean                   remove build/
carry add <pkg>[@<version>]   add a dependency
carry remove <pkg>            remove a dependency
```
