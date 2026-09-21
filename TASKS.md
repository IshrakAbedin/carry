# TASKS — backlog & status

Status: `pending` → `in-progress` → `done` (or `blocked`).
Every task lives in `tasks/Txxx-<slug>.md` (self-contained prompt). Completed tasks have a
result in `results/Txxx.md`. **Next task = first `pending` row in order.**

| ID  | Task | Status  | Result |
|-----|------|---------|--------|
| T000 | Bootstrap: repo-as-memory system, design, plan | done | results/T000.md |
| T001 | Rust CLI scaffold: clap, command stubs, error plumbing | done | results/T001.md |
| T002 | Manifest (CARRY.toml) + `carry new` / `carry init` | done | results/T002.md |
| T003 | Generator: manifest → conanfile.txt + CMakeLists.txt | done | results/T003.md |
| T004 | Build pipeline: conan → cmake → ninja (`carry build`) | done | results/T004.md |
| T005 | `carry run`, `carry test` (ctest), `carry clean` | done | results/T005.md |
| T006 | `carry add` / `carry remove` (toml_edit, format-preserving) | done | results/T006.md |
| T007 | E2E hardening with a real Conan dependency (fmt) | done | results/T007.md |
| T008 | Polish: README, help text, final verification | done | results/T008.md |
| T009 | Demo build with glaze; C++ standard sync (D12) | done | results/T009.md |
| T010 | Compile commands export (compile_commands.json, D13) | done | results/T010.md |

Order: the MVP ran strictly sequential T001 → T008 (frozen at 0.1.0). From T009 on, tasks
are user-directed/ad-hoc. Each task file lists its dependencies and what to read from
earlier results.
