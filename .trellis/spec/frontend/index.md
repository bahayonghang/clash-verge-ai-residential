# Frontend Development Guidelines

This Trellis layer covers the repository's user-facing Clash Verge extension and
local configuration renderer. It is not a browser frontend: there are no UI
components, hooks, styles, TypeScript sources, bundler, or third-party runtime
dependencies.

## Runtime Model

`clash-verge-ai-residential.js` is a pasteable Clash Verge Rev global extension.
Its host entry point receives a Mihomo configuration object and returns the same
object after a validated, idempotent transformation:

```js
function main(config, profileName) {
  if (!config || typeof config !== "object") return config;
  // validate, migrate, and rebuild managed configuration
  return config;
}
```

`scripts/sync-local-config.js` is a separate Node.js 18+ CommonJS CLI. It reads
the public template plus an ignored local TOML file and atomically writes an
ignored `.local.js` file. `tests/regression.test.js` and
`tests/sync-local-config.test.js` exercise both surfaces with plain Node.

## Guidelines Index

| Guide | Repository focus | Status |
|---|---|---|
| [Directory Structure](./directory-structure.md) | Pasteable runtime, tooling, tests, and local artifacts | Complete |
| [Component Guidelines](./component-guidelines.md) | Function and transformation-unit patterns; UI components do not apply | Complete |
| [Hook Guidelines](./hook-guidelines.md) | Clash host entry point, CommonJS CLI boundary, and side effects | Complete |
| [State Management](./state-management.md) | Input configuration ownership, derived state, and idempotence | Complete |
| [Type Safety](./type-safety.md) | Runtime guards and validation in plain JavaScript | Complete |
| [Quality Guidelines](./quality-guidelines.md) | Syntax checks, regression tests, and credential safety | Complete |

## Pre-Development Checklist

- Read `CLAUDE.md` and the guide matching the code being changed.
- Read the relevant section of `clash-verge-ai-residential.js` before changing a
  constant, rule list, DNS policy, or reserved name.
- Search `tests/regression.test.js` for the existing positive, negative,
  migration, and idempotence coverage for that behavior.
- For local rendering changes, read `scripts/sync-local-config.js`,
  `tests/sync-local-config.test.js`, and `docs/local-configuration.md` together.
- Keep the public `HOME_PROXY_TEMPLATE` credentials as `"xxx"` or `""`; never
  edit or commit `*.local.toml` or `*.local.js`.

## Quality Check

Run `just ci` (equivalent to `npm run ci`). It performs `node --check`, the two
plain-Node regression suites, and `scripts/check-template-safety.js`. Domain
changes also require narrow positive coverage and explicit negative coverage for
shared or non-AI traffic. Node tests do not replace a sanitized real-profile
check in Clash Verge Rev when host behavior changes.

Avoid importing component-framework conventions, adding a build step, or moving
the host script behind Node-only APIs. Those changes would break the copy/paste
runtime contract demonstrated by `clash-verge-ai-residential.js` and its guarded
`module.exports` block.
