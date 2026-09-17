# T006 — `carry add` / `carry remove` (toml_edit)

Status: pending
Depends on: T005 (any task that made manifest save real; T002's save is the base)

## Goal

Let users manage dependencies through carry: `carry add <pkg>[@<version>]` and
`carry remove <pkg>` — editing `CARRY.toml` in place with `toml_edit` so formatting and
comments survive, then reporting what changed.

## Context

- Read `DESIGN.md` §4 (version mapping) and `results/T002.md` (manifest load/save).
- `pkg` argument forms: `fmt`, `fmt@10.2.1`, `fmt@>=10` (version part may be a range).
  Parse on the **last** `@`. Package name validated like manifest `name` rule.
- Work in `manifest.rs` (new fns) + `main.rs`.

## Requirements

- `Manifest::add_dep(&mut self, pkg, version) -> Result<()>`:
  - If the dep already exists and version is the same → no-op, print "already up to date".
  - If it exists with a different version → replace, print `updated fmt 10.0 → 10.2.1`.
  - New → print `added fmt 10.2.1`.
  - Save via `toml_edit` doc round-trip: locate/create `[dependencies]` table, set the key,
    keep every other byte/format intact (verify with a diff!). If no `[dependencies]` table
    exists yet, append a clean one at the end with a blank line before it.
- `Manifest::remove_dep(&mut self, pkg) -> Result<()>`: remove the key; error
  `error: dependency 'fmt' not found in CARRY.toml` if absent; drop an empty
  `[dependencies]` table? NO — leave the empty table (stable diff, user may re-add).
- Both commands: after saving, print a hint `run: carry build` (files regenerate on build).
  Do NOT run the build automatically.
- Unit tests: add-new, add-replace, add-same (no write), remove-present, remove-absent,
  and a **formatting-preservation** test — a CARRY.toml with comments and odd spacing keeps
  comments/spacing after add+remove (assert on the raw string).

## Acceptance criteria

- [ ] `cargo test` green incl. the formatting-preservation test; `cargo clippy` clean.
- [ ] Scratch project with a hand-written `CARRY.toml` containing a comment `# my deps`:
      `carry add fmt@10.2.1` → comment still there, diff shows only the new line;
      `carry add fmt@11.0.0` → line updated in place; `carry remove fmt` → key gone,
      `[dependencies]` header remains; `carry remove fmt` again → clean error, exit 1.
- [ ] `carry add` with invalid name (`Bad-Pkg` → allowed by rule? rule is lowercase-only → must error) → error with `help:`.
- [ ] After `carry add fmt@10.2.1`, the next `carry build` regenerates `conanfile.txt`
      containing `fmt/10.2.1` (do NOT run the full conan install here if network is unsure —
      `cargo run -q -- build` may fail at the conan step if offline; that's fine, just
      confirm regeneration happened by inspecting `conanfile.txt` and note the conan result
      in results — network verdict belongs to T007).

## Definition of done

- Criteria + the raw diff outputs recorded in `results/T006.md`; scratch dirs removed.
- TASKS.md/STATE.md/SESSION_LOG.md updated.
