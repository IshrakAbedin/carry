# SESSION LOG — newest on top

Append one entry per session: date, task(s) worked, what was done, verification result,
decisions (pointer to DESIGN.md if any), anything the next session must know.
Keep each entry to ~10 lines max; deep detail belongs in `results/`.

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
