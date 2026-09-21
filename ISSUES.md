# ISSUES — open issues (most severe first)

Format for each issue:

```
## I00N — <title>
Status: open | fixed (by Txxx, date)
Found: <Txxx or session date>
Symptom: <what goes wrong>
Repro: <exact commands>
Notes: <diagnosis, suspected cause, constraints on the fix>
```

Rules:
- File issues found *outside* the current task here instead of fixing them inline.
- One session dedicated to fixing = pick the topmost `open` issue (see prompts/fix-issue.md).
- Never delete entries; flip `Status` to `fixed` and point at the fixing task.

---

## I001 — conan's CMakeUserPresets.json lands in project root, un-ignored
Status: fixed (by fix-issue session 2026-09-21; `GITIGNORE` in src/template.rs now includes `CMakeUserPresets.json`, regression-tested in tests/e2e.rs)
Found: T004 (2026-09-21)
Symptom: After the first `carry build`, conan's CMakeToolchain writes
`CMakeUserPresets.json` into the project **root** (alongside CARRY.toml). The
scaffold `.gitignore` only lists `build/`, so in a git-managed project the
file shows up as an untracked stray.
Repro: `carry new demo && cd demo && carry build && git status` (or just `ls`:
`CMakeUserPresets.json` appears at root, outside `build/`).
Notes: Conan 2.32.0 CMakeToolchain behavior — it always writes the user
presets file next to conanfile/CMakeLists (root), while CMakePresets.json goes
to `build/`. Fix belongs to the scaffold (add `CMakeUserPresets.json` to the
generated `.gitignore` in template.rs) — out of T004 scope. Alternatively a
conan conf could redirect it, but `.gitignore` is the smaller fix. Candidate
for T008 polish or a dedicated session.
