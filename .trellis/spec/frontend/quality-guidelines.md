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

Tests are executable Node scripts with a tiny local harness and
`node:assert/strict`, not a framework:

```js
test("description", () => {
  const rules = buildInjectedRules();
  assert.equal(ruleMatchesHost(rules, "host.example"), false);
});
```

Add focused assertions beside the relevant section in
`tests/regression.test.js`. A new routed domain or regex requires both positive
coverage for the intended AI endpoint and negative coverage for nearby shared,
marketplace, update, CDN, media, advertising, telemetry, or public-DNS traffic.
Migration changes require legacy cleanup and repeated-execution coverage.
Renderer changes require successful-output and rejection coverage in
`tests/sync-local-config.test.js`, including proof that the public template is
unchanged.

## Validation Gate

Run `just ci` before completion. `package.json` defines the exact gate:

1. `npm run check`: `node --check` on the extension, tests, and scripts.
2. `npm test`: the routing regression and local renderer suites.
3. `npm run check:secrets`: `scripts/check-template-safety.js` validates public
   placeholders and recursively scans `.js`, `.json`, `.md`, `.yml`, and
   `.yaml` files outside its excluded directories and local artifacts.

GitHub CI runs the same `npm run ci` on Node 18, 20, and 22. For changes to host
integration, DNS, or routing, also test a sanitized real Clash profile when
practical; the Node suite cannot emulate the Clash JavaScript host or Mihomo.

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
