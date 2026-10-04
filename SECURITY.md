# Security Policy

## Supported versions

Only the latest tagged release and the current `main` branch receive security fixes.

## Reporting a vulnerability

Do not post residential proxy addresses, usernames, passwords, subscription URLs, generated Clash profiles, or unredacted connection logs in a public issue.

For a code-level vulnerability, open a minimal issue that describes the affected version and impact without including secrets. State that sensitive reproduction details are available privately. Repository maintainers can then establish a private channel or enable GitHub Private Vulnerability Reporting.

For an accidental credential commit:

1. Rotate the exposed residential proxy and subscription credentials immediately.
2. Remove the secret from the current branch.
3. Rewrite Git history when the repository has already been pushed or forked.
4. Treat cached workflow logs, artifacts, mirrors, and forks as potentially compromised.

## Scope

Security reports should concern this repository's:

- Clash Verge global extension script, tests, CI, and documentation
- ResiWatch (`residential-monitor`), including the Tauri desktop app and `monitor-db` CLI
- Install scripts (`scripts/nsis-silent-install.js`, `scripts/install-agent-skills.js`, and the ResiWatch NSIS installer)
- The local ResiWatch data directory (`%LOCALAPPDATA%\ResiWatch`)

Connectivity failures caused solely by third-party proxy providers or upstream AI services are operational issues rather than repository vulnerabilities.
