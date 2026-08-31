# Domain docs

How engineering skills should consume this repository's domain documentation when exploring the codebase.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root, or
- **`CONTEXT-MAP.md`** at the repo root if it exists: it points at one `CONTEXT.md` per context. Read each one relevant to the topic.
- Repository directory **`docs/adr/`**: read ADRs that touch the area you are about to work in. In multi-context repos, also check `src/<context>/docs/adr/` for context-scoped decisions. These are Git decision files, not pages on this site. Do not link to `/adr/` or `/en/adr/`.

If any of these files do not exist, **proceed silently**. Do not flag their absence; do not suggest creating them upfront. The `/domain-modeling` skill (reached via `/grill-with-docs` and `/improve-codebase-architecture`) creates them lazily when terms or decisions actually get resolved.

## File structure

Single-context repo (this repository):

```
/
├── CONTEXT.md
├── docs/adr/
│   ├── 0001-process-lookup-vs-process-routing.md
│   ├── 0002-unknown-process-drilldown.md
│   └── 0003-controller-only-process-identity.md
└── src/
```

Multi-context repo (presence of `CONTEXT-MAP.md` at the root):

```
/
├── CONTEXT-MAP.md
├── docs/adr/                          ← system-wide decisions
└── src/
    ├── ordering/
    │   ├── CONTEXT.md
    │   └── docs/adr/                  ← context-specific decisions
    └── billing/
        ├── CONTEXT.md
        └── docs/adr/
```

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `CONTEXT.md`. Do not drift to synonyms the glossary explicitly avoids.

If the concept you need is not in the glossary yet, that is a signal: either you are inventing language the project does not use (reconsider) or there is a real gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts repository file `docs/adr/0001-process-lookup-vs-process-routing.md`, but worth reopening because…_
