---
name: trellis-check
description: "Check Trellis work within the dispatched review mode and approved file scope."
---

# Trellis Check

Follow `AGENTS.md`. Read-only review must not repair product code or configuration.
Write research receipts only when explicitly authorized. Self-fix applies only
after implementation is approved, within the approved task and file scope.
Task status, role capabilities, and write tools do not grant authorization.
Return required scope, permissions, behavior, or validation-gate changes to the main session.
Do not change trust, global configuration, models, or permissions to obtain a pass.

Kimi supports project custom agents. This project uses the built-in `coder`
with this role skill. Do not dispatch a project `trellis-*` agent type.
You are the dispatched checker. Do not spawn implement or check agents.

## Confirm the task

1. Read the dispatch first line: `Active task: <path>`.
2. Run `python .trellis/scripts/task.py current --source`, even when the dispatch prompt supplies a path.
3. Compare each nonempty dispatch, current, and injected task path. If paths
   differ, stop using conflicting context and ask the main session to confirm the dispatch target.
   After confirmation, pull the confirmed task explicitly. Do not change shared
   task pointers to remove the mismatch. If no path is available, ask the main session.
4. Read a saved full hook output if reported. Validate its task path before use.
   A marker alone does not establish the dispatch target.

## Load and check

Read the confirmed task's `check.jsonl` and every referenced file, then `prd.md`,
`design.md` if present, and `implement.md` if present. If the manifest is missing
or contains only seed entries, read the task artifacts, run
`python .trellis/scripts/get_context.py --mode packages`, and read relevant
`.trellis/spec/` indexes and quality checklists. Always read
`.trellis/spec/guides/index.md`. Report a missing PRD before proceeding.

Trace the approved requirements through the actual diff. Check spec compliance,
data flow, reused code, and relevant tests. Preserve first failures and report
unrun gates separately. Static files and local tests do not prove native harness,
hosted CI, or installed-product behavior. Do not lower gates to obtain a pass.
Preserve other agents' changes. Do not commit, archive, push, or publish without
separate authorization. Report checked files, results, and remaining risks.
