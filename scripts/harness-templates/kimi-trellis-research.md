---
name: trellis-research
description: "Research a confirmed Trellis task and save evidence within its research directory."
---

# Trellis Research

Follow `AGENTS.md`. Product code, specs, platform configuration, and Git operations
remain read-only. Write only explicitly authorized evidence in the confirmed
task's `research/` directory. Tools and task status do not grant additional permission.
Do not change trust, global configuration, permissions, or models.

Kimi supports project custom agents. This project uses the built-in `coder`
with this role skill because the research role must save evidence. The built-in
`explore` role does not provide the required write access. Do not dispatch a
project `trellis-*` agent type or spawn implement or check agents.

## Confirm the task

1. Read the dispatch first line: `Active task: <path>`.
2. Run `python .trellis/scripts/task.py current --source`, even when the dispatch prompt supplies a path.
3. Compare each nonempty dispatch, current, and injected task path. If paths
   differ, stop using conflicting context and ask the main session to confirm the dispatch target.
   After confirmation, pull the confirmed task explicitly. Do not change shared
   task pointers to remove the mismatch. If no path is available, ask the main session.
4. Read a saved full hook output if reported and validate the task path before use.

## Research and report

Read `prd.md`, `design.md` if present, and relevant approved research material.
Do not read `implement.jsonl` or `check.jsonl`. Run
`python .trellis/scripts/get_context.py --mode packages` and read the relevant
spec indexes and `.trellis/spec/guides/index.md`. Report a missing PRD.

Search before drawing conclusions. Record actual paths, code evidence, versions,
sources, uncertainty, and untested behavior. Accept user failure reports as initial
evidence. Reproduce only when reproduction locates the cause or validates a fix.
Do not copy private configuration, credentials, databases, or history into receipts.
Save findings only under the confirmed task's `research/`. Report proposed product
changes to the main session. Do not repair product files or expand the approved scope.
