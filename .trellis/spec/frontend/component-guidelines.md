# Runtime Function Guidelines

This repository has no visual components or props. The reusable units are plain
functions that validate, derive, or merge Clash/Mihomo configuration.

## Function Shape

- Keep leaf helpers small and deterministic where practical. Examples include
  `isPlainObject`, `uniqueStrings`, `resolveCandidate`, and
  `buildNameserverPolicy` in `clash-verge-ai-residential.js`.
- Normalize untrusted host shapes at the boundary before iterating. Existing
  helpers use `Array.isArray`, `typeof`, and plain-object checks rather than
  assuming subscription data is well formed.
- Use builders for derived arrays and objects, then let `main` orchestrate them.
  Do not bury the entire transformation in the entry point.
- Preserve the explicit fail-closed behavior for ambiguous names, recursive
  proxy graphs, invalid credentials, and missing upstreams.

The local defensive pattern is concise:

```js
function cloneObject(value) {
  return isPlainObject(value) ? { ...value } : {};
}

function toStringArray(value) {
  if (typeof value === "string") return value.length > 0 ? [value] : [];
  if (!Array.isArray(value)) return [];
  return value.filter((item) => typeof item === "string" && item.length > 0);
}
```

## Parameters And Return Values

Use positional parameters for small transformations (`buildDnsConfig(existingDns,
upstreamName)`) and an options object when several paths have defaults
(`syncLocalConfig({ templatePath, configPath, outputPath })`). Return the
transformed value or a small result object; throw a specific `Error` when a
safety invariant cannot be met.

`main(config, profileName)` intentionally mutates the received top-level config
and returns it because that is the Clash extension contract. Helpers that rebuild
nested sections usually clone or allocate (`buildDnsConfig`, `upsertNamedItem`)
so they do not accidentally retain incompatible fields.

## Composition And Portability

The root script must remain executable without Node globals. Its CommonJS export
is guarded solely for tests:

```js
if (typeof module !== "undefined" && module.exports) {
  module.exports = { main, buildDnsConfig, buildInjectedRules, constants: {} };
}
```

Keep code comments and error messages in Chinese, matching the source. Preserve
2-space indentation, double quotes, semicolons, and `"use strict"`.

Styling, DOM composition, and accessibility component APIs do not apply because
the project renders no interface. User-visible behavior is the generated Clash
configuration and CLI output.

## Anti-Patterns

- Do not add framework components, dependency injection, classes, or abstraction
  layers that have no repeated source pattern.
- Do not silently coerce an unsafe configuration into a usable proxy chain.
- Do not mutate shared policy tables as part of normal runtime execution.
- Do not call Node-only APIs from the root extension or export unguarded
  CommonJS symbols into the Clash host.
