# Quality Guidelines

## Required Style And Compatibility

- Target Node.js 18+ for repository scripts and tests while keeping
  `clash-verge-ai-residential.js` compatible with the Clash Verge Rev extension
  host.
- Use zero third-party dependencies, CommonJS, `"use strict"`, 2-space
  indentation, double quotes, and semicolons. There is no formatter or linter;
  match the surrounding source and rely on syntax checks.
- Keep code comments, error messages, and `docs/` in Chinese. Keep this Trellis
  spec, `package.json`, CI, and GitHub templates in English.
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
Managed-rule ownership changes require current-output cleanup, unknown/retired
rule preservation, and repeated-execution coverage. Renderer changes require
successful-output and rejection coverage in `tests/sync-local-config.test.js`,
including proof that the public template is unchanged and failed validation
does not create a partial output. Generated-script behavior should be probed in
a separate Node process when a public default is overridden.

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
- Do not add dependencies or build tooling for behavior the standard library
  already supports.
