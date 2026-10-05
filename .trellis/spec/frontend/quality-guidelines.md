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
- Keep code comments, error messages, `CHANGELOG.md`, and `docs/` (except
  `docs/en/`) in Chinese. `docs/en/` is the English docs-site tree. Keep this
  Trellis spec, `package.json`, CI, and GitHub templates in English.
- Preserve the AI-only routing boundary and the generated singleton residential
  egress group. Configuration structure does not establish host fail-closed
  behavior: the Clash host may discard a failed script and use its input profile.

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
Routing and nameserver-policy may be decoupled. An extra site with
`residentialDns: false` still injects `DOMAIN` / `DOMAIN-SUFFIX` rules when
its category and site switches are on, but `buildNameserverPolicy` must not
write `+.${domain}` or the bare hostname for that site. Existing non-extra
AI domains keep residential DNS. Extra sites live in `EXTRA_SITES`; do not
bind every new suffix to residential DoH. Node tests must not claim a
third-party host completed TLS or is available; they only assert generated
rules and policy keys.
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

Run `just ci` before completion. `just ci` is `monitor-check` followed by root
`npm run ci`. `just ci` is not equal to `npm run ci` alone and does not build
docs. The monitor gate includes version checks, its frontend install and
`npm run check` (typecheck, lint, test, build), and Rust fmt/clippy/tests.
Root `package.json` defines:

1. `npm run check`: `node --check` on the extension, all tests, and scripts,
   then `npm run check:agents` for deterministic agent-contract checks.
2. `npm test`: explicitly listed `node:test` suites for routing, the local
   renderer, template safety, and agent-contract positive/negative fixtures.
3. `npm run check:secrets`: `scripts/check-template-safety.js` validates public
   placeholders and recursively scans `.js`, `.json`, `.jsonl`, `.md`, `.py`,
   `.toml`, `.yml`, and `.yaml` files outside its excluded directories and local
   artifacts.

GitHub CI runs matrix `npm run ci` on Ubuntu with Node 18, 20, and 22, plus
Windows with Node 22; a Windows monitor job with seven separate pwsh native
steps (install, full npm audit, frontend check, Rust fmt, clippy, workspace
tests, secret scan); and an Ubuntu Node 22 docs job (`npm --prefix docs ci`,
independent `npm --prefix docs audit --include=dev --audit-level=high`, then
`npm --prefix docs run build`). The aggregate job named `Required checks`
needs `[test, monitor, docs]`. Branch protection depends only on that stable
aggregate job. The VitePress docs toolchain is Node.js 22+; local docs build
is `just docs-build` and is independent of `just ci`. For changes to host
integration, DNS, or routing, also test a sanitized real Clash profile when
practical; the Node suite cannot emulate the Clash JavaScript host or Mihomo.

Run the separate `just dependency-audit` for dependency/lockfile or audit-gate
changes. The recipe runs `npm --prefix residential-monitor audit --include=dev
--audit-level=high`, `npm --prefix docs audit --include=dev --audit-level=high`,
then `cargo audit --file residential-monitor/src-tauri/Cargo.lock`. Full npm
audits include development dependencies even when npm configuration omits
them. The recipe requires network access and an installed `cargo-audit`.
Missing tools and registry/database/network failures are blocked checks, not
vulnerability findings or PASS. Record RustSec warnings and affected targets
separately. Hosted CI does not run RustSec audit. A successful install does
not satisfy this independent security gate.

`scripts/check-agent-contract.js` uses only Node standard-library APIs. It
checks shared entry points, named gates, optional local overrides when present, local
task references, workflow tag structure, and explicit authorization markers.
Its fixtures must reject missing contracts, reverse imports, gate drift,
invalid checked markers in present overrides, stale task paths, and missing authorization boundaries.
Fixtures must accept a checkout without the four ignored local overrides.
The checker does not prove semantic agreement or client runtime behavior;
strong-model review and fresh-session evidence remain separate requirements.
These contracts apply to Claude Code, Codex, Grok Build, Kimi Code, and OMP.

## Scenario: Optional Local Harness Overrides

### 1. Scope / Trigger

Apply the user-approved 2026-10-01 policy to Codex and Kimi local overrides.
The repository ignores these files. A new checkout does not include them.

### 2. Signatures

- `node scripts/check-agent-contract.js` validates shared contracts and present overrides.
- `node scripts/check-harness-environment.js` reports override presence and hashes.
- A matching Trellis entry runs `init --claude --codex --grok --kimi --omp --skip-existing -y` only in an authorized isolated directory.

### 3. Contracts

The optional files are `.codex/config.toml` and
`.kimi-code/skills/trellis-{implement,check,research}/SKILL.md`.
Copy selected local overrides before isolated bootstrap only within existing
approval. Record copied-file hashes before and after init. Do not add the files
to Git or create missing
overrides during a contract check. Missing overrides do not prove role loading.
Init can generate default files at the optional paths. Run the contract checker
after init and validate the generated files as present overrides. Init exit 0
does not establish authorization compliance or native role loading.

### 4. Validation & Error Matrix

- Missing local override: accept the contract check; report absence in diagnostics.
- Present Codex config or Kimi check role violates checked markers: fail the contract check.
- Kimi implement/research role: check readability; do not claim validation of role contents.
- Copied override bytes change during `--skip-existing` init: fail preservation.
- No matching Trellis entry: record `BLOCKED`; do not run init.
- Init exit 0 with invalid generated override markers: record `INIT_PASS / CONTRACT_FAIL`; preserve the first failure and generated bytes.

### 5. Good / Base / Bad Cases

Good: copied local overrides retain their hashes. Base: a clean checkout has
no local overrides. Bad: report successful native role loading from a missing file.

### 6. Tests Required

Use existing positive and negative contract fixtures for absent overrides and
invalid checked markers. Check diagnostics for presence and hashes. Keep bootstrap
byte preservation separate from client fresh-session evidence. Preserve the
actual missing-override init receipt and its post-init contract result; do not
replace generated defaults with local overrides to hide a failed check.

### 7. Wrong vs Correct

Wrong: require ignored overrides in every checkout. Correct: validate present
overrides and report absent overrides without changing the working tree.

For core routing changes, compare the sanitized default projection against
`tests/fixtures/routing-default-v5.11.json`, sourced from its recorded baseline
commit rather than the edited implementation. Only the contiguous AI domain-rule
block may be sorted; private, IP, process and original Profile rule order remains
significant. Normalize DNS object keys, never resolver-array order. Also test
core on/off/on, full old-rule cleanup and the dedicated process/IP gate matrix.
The existing regex routes also use the reserved inline/classical DNS provider
`AI-家宽-DNS-REGEX`. Keep the historical fixture unchanged and declare the exact
DNS policy addition in the default-projection test; assert provider contents
separately. Test Vertex/Cursor gating, cleanup, ownership and negative host
matches. Never widen Google/Cursor suffixes, use raw regex DNS keys, or infer
real host behavior from Node tests. Record actual core DNS/proxy observations;
loopback fixtures do not establish public DoH TLS or production UDP behavior.

## Scenario: Isolated Harness Bootstrap

### 1. Scope / Trigger

Use the project bootstrap entry when a new isolated candidate needs the shared
contract and four public default overrides. Keep environment diagnostics read-only.

### 2. Signatures

`node scripts/bootstrap-harnesses.js --root <isolated-temp-candidate> --entry <absolute-js-or-exe>`

### 3. Contracts

Use the same selected entry for version and five-platform init. Require the
project `.trellis/.version`. Copy only the named public input set. Deploy public
templates from `scripts/harness-templates/` before init only when their target
files are absent. Preserve present override bytes. Run the unchanged candidate
contract checker after init. Do not change current local overrides, global
tools, trust, models, or permissions. The target is a minimum contract candidate.

### 4. Validation & Error Matrix

- Wrong version or unavailable entry: fail before init.
- Current checkout, disallowed target, link boundary, or wrapper entry: reject.
- Existing override: preserve bytes; a contract error remains a failure.
- Init or checker nonzero: record separate failure; preserve output.
- Completed target reused: reject without rewriting files.

### 5. Good / Base / Bad Cases

Good: missing overrides receive public defaults and pass the real checker.
Base: existing valid overrides retain their hashes. Bad: replace an invalid
existing override to report success.

### 6. Tests Required

Fixtures must cover selected-entry version checks, missing-file deployment,
present-file byte preservation, target rejection, native failure propagation,
and a real checker run over public templates. Record native isolated init
separately from injected fixture execution and client role loading.

Basic client startup and public-read checks may run with recorded competing
workload when the user authorizes that condition. Record actual read events,
protocol errors, native exit, driver exit, and capture stop separately. Do not
transfer basic startup results to quantitative performance gates, hook loading,
permission enforcement, or complete child lifecycle acceptance. For Kimi 2.0.0
ACP plan checks, require the mode response and mode update before prompt;
cancel permission requests and reject callbacks outside the named public files.

### 7. Wrong vs Correct

Wrong: run broad init in the current checkout. Correct: explicitly select a new
isolated candidate and preserve existing overrides.

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
  "block_creations": false,
  "required_conversation_resolution": true,
  "lock_branch": false,
  "allow_fork_syncing": false
}
```

Omit `contexts` and `required_signatures` from this PUT. `contexts` is
GET-derived from `checks`. Signatures use a separate endpoint. Include the
boolean flags that the GET reports so an incomplete PUT does not clear them.

### 8. Live verification

Date: 2026-09-07.

Independent `GET /repos/bahayonghang/clash-verge-ai-residential/branches/main/protection`
on 2026-09-07 recorded `required_linear_history.enabled=true`. Other protection
fields matched the approved contract: `required_status_checks.strict=true` with
app-bound `Required checks` (`app_id` 15368), pull-request reviews with
`required_approving_review_count=0`, `dismiss_stale_reviews=false`,
`require_code_owner_reviews=false`, `require_last_push_approval=false`,
`enforce_admins.enabled=true`, `required_conversation_resolution.enabled=true`,
`allow_force_pushes.enabled=false`, `allow_deletions.enabled=false`,
`required_signatures.enabled=false`, `block_creations.enabled=false`,
`lock_branch.enabled=false`, and `allow_fork_syncing.enabled=false`. GET still
derives `contexts: ["Required checks"]` from `checks`. A later independent GET
on the same date returned the same comparable fields. This section records those
GETs. The local file is not the remote setting.

The next ordinary PR exact-head `Required checks` evidence is UNVERIFIED. This
change did not open a test PR.

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
- Confirm new domains have official or sanitized connection evidence, a
  real TLS check at the intended DNS vantage, and a narrow negative-scope
  analysis, matching `.github/pull_request_template.md`. CDN/GeoDNS hosts
  can fail when residential DoH sees a different edge than airport-upstream
  DoH; do not assume `DOMAIN-SUFFIX` and residential DNS must be paired.
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
