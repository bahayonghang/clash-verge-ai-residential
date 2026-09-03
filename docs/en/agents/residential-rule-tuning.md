# Residential rule-tuning skill

Source: `skills/residential-rule-tuning/`.

Install:

```bash
just install-all
just install-cli
just install-skills
just install-skills --check
just install-skills --force
```

`just install-all` (Windows) silently installs the desktop app to `%LOCALAPPDATA%\ResiWatch`, installs `monitor-db` into the cargo user bin, and writes the skill into this project's `.agents/skills` and `.claude/skills` (creating those directories if needed).

`just install-skills` writes only into platform directories that already exist. It does not create missing platforms and does not change other skills. If the target has a same-name file with different content, the default is fail closed. `--force` writes `<name>.bak-<UTC>` first, then replaces.

Companion CLI: `just monitor-db --help`. Queries default to JSON. Add `--redact` before pasting output.
