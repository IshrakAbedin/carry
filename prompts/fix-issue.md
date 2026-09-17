# Prompt: fix the top open issue (paste into a fresh session)

You are working on the "carry" project in this repository. Your context is empty — the repo
is your memory. Read `BOOTSTRAP.md` first for the recording protocol and hard rules.

1. Read `ISSUES.md` and pick the topmost issue with `Status: open`.
2. Reproduce it exactly as written (record the real output).
3. Fix it at the root cause — no band-aids that bypass safety checks or paper over symptoms.
   If the fix is large, split it: fix the minimal correct version now and file a follow-up
   task in `TASKS.md` for the rest.
4. Add a regression test where feasible (unit or `#[ignore]`d E2E per the project's test
   policy in DESIGN.md §7).
5. Verify the project stays green: `cargo build`, `cargo test`, `cargo clippy --all-targets`.
6. Record: flip the issue to `fixed (by <where fixed>, <date>)` in ISSUES.md, append a
   `SESSION_LOG.md` entry, and note it in `STATE.md` if it changed the baseline.

If there are no open issues, say so and stop — do not invent work.
