# Quality Guidelines

## Required Style And Compatibility

- Target Node.js 18+ for repository scripts and tests while keeping
  `clash-verge-ai-residential.js` compatible with the Clash Verge Rev extension
  host. The VitePress docs site in `docs/package.json` requires Node.js 22+ and
  is a separate toolchain.
- Use zero third-party dependencies, CommonJS, `"use strict"`, 2-space
  indentation, double quotes, and semicolons for the extension and root
  scripts. There is no formatter or linter; match the surrounding source and
  rely on syntax checks.
- Keep code comments, error messages, and `docs/` (except `docs/en/`) in
  Chinese. `docs/en/` is the English docs-site tree. Keep this Trellis spec,
  `package.json`, CI, and GitHub templates in English.
- Preserve the AI-only routing boundary and fail-closed proxy-chain behavior.

## Test Pattern

Tests use the built-in `node:test` runner with `node:assert/strict`, not a
third-party framework:

```js
const assert = require("node:assert/strict");
const { test } = require("node:test");

test("description", () => {
  const rules = buildInjectedRules();
  assert.equal(ruleMatchesHost(rules, "host.example"), false);
});
```

Add focused assertions beside the relevant section in
`tests/regression.test.js`. A new routed domain or regex requires both positive
coverage for the intended AI endpoint and negative coverage for nearby shared,
marketplace, update, CDN, media, advertising, telemetry, or public-DNS traffic.
`buildNameserverPolicy` writes suffix domains as `+.${domain}` and exact
domains as the bare hostname. DNS on/off assertions must use those keys.
A bare suffix name such as `chatgpt.com` is absent even when the route is
on, so `host in policy` cannot prove that GPT DNS routing is disabled.
Managed-rule ownership changes require current-output cleanup, unknown/retired
rule preservation, and repeated-execution coverage. Renderer changes require
successful-output and rejection coverage in `tests/sync-local-config.test.js`,
including proof that the public template is unchanged and failed validation
does not create a partial output. Generated-script behavior should be probed in
a separate Node process when a public default is overridden.
`SWITCH_CONFIG_FIELDS` rows must appear as markdown table cells in both
`docs/configuration.md` and `docs/local-configuration.md` (`| \`table.key\` | \`CONSTANT\` | \`default\` |`).
Removing the table from `docs/configuration.md` fails that test even if
`docs/local-configuration.md` still has it.

## Validation Gate

Run `just ci` before completion. `package.json` defines the exact gate:

1. `npm run check`: `node --check` on the extension, all tests, and scripts.
2. `npm test`: explicitly listed `node:test` suites for routing, the local
   renderer, and template safety.
3. `npm run check:secrets`: `scripts/check-template-safety.js` validates public
   placeholders and recursively scans `.js`, `.json`, `.jsonl`, `.md`, `.py`,
   `.toml`, `.yml`, and `.yaml` files outside its excluded directories and local
   artifacts.

GitHub CI runs the same `npm run ci` on Ubuntu with Node 18, 20, and 22, plus
Windows with Node 22. Branch protection depends only on the stable
`Required checks` aggregate job. For changes to host integration, DNS, or
routing, also test a sanitized real Clash profile when practical; the Node
suite cannot emulate the Clash JavaScript host or Mihomo.

## Scenario: Main Branch Protection

### 1. Scope / Trigger

Apply this contract whenever `.github/workflows/ci.yml`, the stable required
job name, or GitHub `main` branch protection changes.

### 2. Signatures

- Read checks: `GET /repos/{owner}/{repo}/commits/{sha}/check-runs`
- Apply protection: `PUT /repos/{owner}/{repo}/branches/main/protection`
- Verify protection: `GET /repos/{owner}/{repo}/branches/main/protection`

### 3. Contracts

- Discover the successful `Required checks` run on the current `main` SHA and
  bind protection to its GitHub App `app_id`; do not hardcode an unverified app.
- Send `required_status_checks.strict = true` with app-bound `checks` containing
  exactly `Required checks`.
- Require pull requests with zero approvals for the single-maintainer repository,
  enforce administrators, resolve conversations, and require linear history.
- Disable force pushes and branch deletion. Do not add deployment, CODEOWNERS,
  bypass-actor, or other-branch requirements without a separate decision.

### 4. Validation & Error Matrix

- Missing or unsuccessful `Required checks` on `main` -> stop before protection.
- Environment PAT returns `403` -> clear `GH_TOKEN`/`GITHUB_TOKEN` for the
  process and use the authenticated GitHub CLI keyring credential.
- `required_status_checks` includes both legacy `contexts` and app-bound
  `checks` -> GitHub may return `422`; omit `contexts` from the PUT request.
- The GET response may derive a legacy `contexts` list from `checks`; verify its
  names, but use `checks` and `app_id` as the authoritative app binding.
- Any GET field differs from the approved contract -> fail verification and
  inspect actual state before sending a corrective full request.

### 5. Good / Base / Bad Cases

- Good: all matrix jobs and `Required checks` pass on `main`, then protection is
  applied and independently read back.
- Base: a PR is blocked while checks are pending and becomes mergeable after the
  current head SHA passes the stable gate.
- Bad: direct administrator push, force push, deletion, a stale branch, or a
  same-name check from an unbound app satisfies the policy.

### 6. Tests Required

- Assert the PR head SHA before merge and use `--match-head-commit`.
- Assert all PR checks succeed, then assert the `main` push run succeeds.
- Assert the protection GET response covers strict/app-bound checks, PR count,
  administrators, conversations, linear history, force pushes, and deletion.
- Complete the Trellis closeout through a protected PR to prove the maintenance
  workflow remains usable without an administrator bypass.

### 7. Wrong vs Correct

Wrong `required_status_checks` fragment: legacy and app-bound selectors are
mixed.

```json
{
  "strict": true,
  "contexts": [],
  "checks": [
    {"context": "Required checks", "app_id": 15368}
  ]
}
```

Correct PUT body after discovering `app_id` from the successful check run:

```json
{
  "required_status_checks": {
    "strict": true,
    "checks": [
      {"context": "Required checks", "app_id": 15368}
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": false,
    "require_code_owner_reviews": false,
    "required_approving_review_count": 0,
    "require_last_push_approval": false
  },
  "restrictions": null,
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_conversation_resolution": true
}
```

## Security And Generated Files

`HOME_PROXY_TEMPLATE.server`, `.username`, and `.password` must remain `"xxx"`
or `""` in the public root script. Never commit subscription URLs, credentials,
generated profiles, local TOML, `.local.js`, or unredacted Connections logs.
Generate local output through `just render-local`; do not hand-edit it.

## Review Checklist

- Search every changed constant or managed name across source, tests, docs,
  migration sets, and generated-template handling.
- Confirm rule order, DNS policy, upstream recursion checks, and idempotence remain
  coherent across `main`.
- Confirm new domains have official or sanitized connection evidence and a
  narrow negative-scope analysis, matching `.github/pull_request_template.md`.
- Confirm public placeholders and ignored-file boundaries remain intact.
- Run `just ci` and inspect the final diff for unrelated or generated files.

Accessibility and visual-browser checks do not apply because the repository has
no rendered UI.

## Anti-Patterns

- Do not add broad provider suffixes or route shared infrastructure by default.
- Do not weaken a failing validation into a warning when it protects credentials,
  name uniqueness, proxy recursion, UDP capability, or upstream selection.
- Do not write tests that only repeat a constant; assert observable generated
  rules/configuration and explicit exclusions.
- Do not assert `host in nameserver-policy` for a suffix domain. Check
  `+.${host}` (suffix) or the bare hostname (exact).
- Do not add dependencies or build tooling for behavior the standard library
  already supports.
