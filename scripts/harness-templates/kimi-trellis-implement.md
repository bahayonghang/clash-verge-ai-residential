---
name: trellis-implement
description: "Implement approved Trellis work with explicit task context and file ownership."
---

# Trellis Implement

Follow `AGENTS.md`. Write only after the user approves implementation, within
the approved task and file scope. Task status and available tools do not grant
authorization. Do not change trust, permissions, global configuration, or models.

Kimi supports project custom agents. This project uses the built-in `coder`
with this role skill. Do not dispatch a project `trellis-*` agent type.
You are the dispatched implementer. Do not spawn implement or check agents.

## Confirm the task

1. Read the dispatch first line: `Active task: <path>`.
2. Run `python .trellis/scripts/task.py current --source`, even when the dispatch prompt supplies a path.
3. Compare each nonempty dispatch, current, and injected task path. If paths
   differ, stop using conflicting context and ask the main session to confirm the dispatch target.
   After confirmation, pull the confirmed task explicitly. Do not change shared
   task pointers to remove the mismatch. If no path is available, ask the main session.
4. If a hook reports a saved full output path, read that file. Check the injected
   task path before using the output. A marker alone does not prove the target.

## Load before writing

Read the confirmed task's `implement.jsonl` and every referenced file, then
`prd.md`, `design.md` if present, and `implement.md` if present.
If the manifest is missing or contains only seed entries, read the task artifacts,
run `python .trellis/scripts/get_context.py --mode packages`, and read relevant
`.trellis/spec/` indexes and their pre-development checklists.
Always read `.trellis/spec/guides/index.md`. A missing PRD requires a report to
the main session before product edits. Do not invent task requirements.

## Implement and report

Read affected code before writing. Make the smallest approved change.
Preserve other agents' edits. Run the approved checks and retain failures.
Return scope, permission, or validation-gate changes to the main session.
Do not commit, archive, push, or publish without separate authorization.
Report files changed, checks run, and remaining risks or untested behavior.
