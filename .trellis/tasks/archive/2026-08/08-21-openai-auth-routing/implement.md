# Implementation Plan

## Preconditions And Scope Guard

- Do not run `task.py start` until the user explicitly approves the latest
  planning summary in a subsequent message.
- Recheck `git status --short` before editing and preserve the unrelated
  `residential-monitor/src-tauri/Cargo.toml` change.
- Never print, stage, or commit the ignored local TOML or generated local JS.
- Keep the public `HOME_PROXY_TEMPLATE` placeholders unchanged and do not alter
  the airport-to-residential chain.

## Ordered Work

### 1. Add Focused Failing Coverage

- [x] Extend `tests/regression.test.js` with public-default, auth-only,
      web-assets-only, both-enabled, negative-scope, DNS symmetry, cleanup, and
      second-run idempotency assertions.
- [x] Use isolated temporary patched/generated modules rather than mutating the
      checked-in defaults in-process.
- [x] Extend `tests/sync-local-config.test.js` so mapping-driven coverage sees
      both new keys, partial TOML completion keeps existing content, and a
      generated `openai_auth = true` script changes observable route/DNS output.
- [x] Include explicit negatives for `www.openai.com`, an unrelated
      `openai.com` child, `oaistatsig.com`, `oaistatic.com` in auth-only mode,
      and first-party auth hosts in assets-only mode.

### 2. Implement Runtime Catalogs And Ownership

- [x] Add `ROUTE_OPENAI_AUTH = false` and
      `ROUTE_OPENAI_WEB_ASSETS = false` in the bounded public configuration
      section.
- [x] Add/export the three domain catalogs from the technical design without
      adding a generic `openai.com` suffix.
- [x] Wire the catalogs into active suffix/exact builders under their own
      switches.
- [x] Wire the catalogs unconditionally into `allPossibleSuffixDomains()` and
      `allPossibleExactDomains()` for current-managed cleanup.
- [x] Confirm rule generation and nameserver-policy generation remain derived
      from the same active catalogs; do not add a parallel DNS-only list.
- [x] Update header/runtime version to 5.11.0 and adjust only version-dependent
      root tests.

### 3. Extend The TOML Contract

- [x] Add the two rows to `SWITCH_CONFIG_FIELDS` using keys
      `openai_auth` and `openai_web_assets`.
- [x] Add both keys as `false` to
      `clash-verge-ai-residential.local.toml.example` in the OpenAI routing block.
- [x] Keep parser grammar, exact-one injection, atomic output, and completion
      algorithms unchanged unless a focused test exposes a real gap.
- [x] Run completion against a temporary legacy/partial fixture and confirm
      values, comments, credentials, CRLF/LF, and trailing-newline behavior.

### 4. Apply The Authorized Local Values

- [x] Run the renderer so the ignored current TOML receives missing keys using
      tracked example defaults.
- [x] Change only `routing.openai_auth` to `true`; leave
      `routing.openai_web_assets`, `routing.antigravity_google_auth`, and
      `routing.ai_process_fallback` false.
- [x] Render the ignored local JS again.
- [x] Verify only the four selected boolean constants and representative
      auth/asset rule behavior. Do not output home-proxy fields or credentials.
- [x] Confirm both local artifacts remain ignored and absent from the Git diff.

### 5. Synchronize Documentation And Release Metadata

- [x] Add 5.11.0 Added/Notes entries to CHANGELOG and synchronize root
      `package.json`, README, script header, and version regression assertion.
- [x] Add both switches to `docs/configuration.md` and
      `docs/local-configuration.md` with exact key, constant, default, domain
      scope, and independence from shared dependencies.
- [x] Rewrite the OpenAI authentication-exit section in
      `docs/routing-scope.md` to describe default split, first-party auth
      opt-in, remaining third-party split, and `UNVERIFIED` host evidence.
- [x] Add a concise Windows-to-Ubuntu deployment note to
      `docs/local-configuration.md`: copy the rendered `.local.js`, not the
      TOML; disclose that the rendered script itself embeds the endpoint and
      credentials and therefore requires trusted transfer and restricted read
      access; require the same resolvable upstream group and record Ubuntu host
      execution as a manual check.
- [x] Do not change monitor versions, Google auth defaults, chain guidance, or
      unrelated Unreleased monitor notes.

### 6. Verify And Review

- [x] Run syntax checks for the root extension, renderer, and changed tests.
- [x] Run focused routing and renderer suites, then the repository secret scan.
- [x] Run the complete `just ci` gate, which covers the Windows/Ubuntu-supported
      Node contract; keep actual Clash host behavior separately `UNVERIFIED`.
- [x] Run `git diff --check`, inspect the complete scoped diff, and confirm the
      unrelated Cargo edit is byte-for-byte untouched by this task.
- [x] Run `task.py validate` for the active task artifacts.

## Validation Commands

```powershell
node --check clash-verge-ai-residential.js
node --check scripts/sync-local-config.js
node --check tests/regression.test.js
node --check tests/sync-local-config.test.js
node --test tests/regression.test.js
node --test tests/sync-local-config.test.js
npm run check:secrets
just ci
git diff --check
python .trellis/scripts/task.py validate .trellis/tasks/08-21-openai-auth-routing
```

Also run a credential-safe local probe that reports only:

- `ROUTE_OPENAI_AUTH`;
- `ROUTE_OPENAI_WEB_ASSETS`;
- `ROUTE_ANTIGRAVITY_GOOGLE_AUTH`;
- `ENABLE_AI_PROCESS_FALLBACK`;
- boolean matches for representative auth and asset hosts.

Expected selected local values are `true`, `false`, `false`, and `false` in
that order. Do not display the home proxy object.

## Completion Evidence Boundary

- Passing Node and CI checks prove the repository contract and cross-platform
  renderer behavior.
- They do not prove an actual ChatGPT login, provider risk-control outcome,
  Clash Verge Rev host execution, or a single exit across third-party redirects.
- Report the real login-flow check as `UNVERIFIED` unless sanitized Connections
  evidence is actually collected.

## Commit Boundary

Do not commit, push, archive, or modify task status beyond `in_progress` without a
separate explicit user request after implementation and verification.

## Execution Record - 2026-08-21

- Implemented the 5.11.0 runtime catalogs, independent public switches,
  renderer mapping/example, managed rule and DNS cleanup, tests, and docs.
- Added behavioral true-to-false cleanup coverage that preserves unknown
  user-authored rules while removing managed rules and DNS keys.
- Updated the ignored current TOML to `openai_auth = true` and
  `openai_web_assets = false`; regenerated the ignored local script. Safe probes
  confirmed Google auth and process fallback remain false without printing
  proxy endpoint or credential fields.
- Focused routing tests passed 51/51 and renderer tests passed 15/15. The final
  `just ci`, secret scan, `git diff --check`, and task validation all passed.
- The pre-existing `residential-monitor/src-tauri/Cargo.toml` SHA-256 remained
  `572F347BBE87A0A095892BA616074C433A90D7E3F557AA49653344A72F5EEE8F`.
- Actual Ubuntu Clash host execution and end-to-end ChatGPT login behavior
  remain `UNVERIFIED` pending sanitized Connections evidence.
- No commit, push, archive, or task completion action was performed.
