# TASKS — backlog & status

Status: `pending` → `in-progress` → `done` (or `blocked`).
Every task lives in `tasks/Txxx-<slug>.md` (self-contained prompt). Completed tasks have a
result in `results/Txxx.md`. **Next task = first `pending` row in order.**

| ID  | Task | Status  | Result |
|-----|------|---------|--------|
| T000 | Bootstrap: repo-as-memory system, design, plan | done | results/T000.md |
| T001 | Rust CLI scaffold: clap, command stubs, error plumbing | done | results/T001.md |
| T002 | Manifest (CARRY.toml) + `carry new` / `carry init` | pending | — |
| T003 | Generator: manifest → conanfile.txt + CMakeLists.txt | pending | — |
| T004 | Build pipeline: conan → cmake → ninja (`carry build`) | pending | — |
| T005 | `carry run`, `carry test` (ctest), `carry clean` | pending | — |
| T006 | `carry add` / `carry remove` (toml_edit, format-preserving) | pending | — |
| T007 | E2E hardening with a real Conan dependency (fmt) | pending | — |
| T008 | Polish: README, help text, final verification | pending | — |

Order: strictly sequential T001 → T008. Each task file lists its dependencies and what to
read from earlier results.
