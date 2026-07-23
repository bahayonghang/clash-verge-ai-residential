# Type Safety

The project uses plain JavaScript, not TypeScript or a schema library. Safety
comes from explicit runtime validation at host, parser, and filesystem
boundaries. Do not describe static types that the repository does not have.

## Runtime Guards

Use the existing guard style before reading arrays or objects:

```js
function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

for (const rule of Array.isArray(rules) ? rules : []) {
  if (typeof rule === "string" && managed.has(rule)) continue;
}
```

Hyphenated Mihomo keys use bracket notation, for example
`config["proxy-groups"]`, `homeProxy["dialer-proxy"]`, and
`dns["nameserver-policy"]`. Preserve scalar distinctions when deduplicating;
`uniqueScalars` keys values by both `typeof` and string value.

## Boundary Validation

- `main` accepts only an object-like configuration and normalizes absent arrays.
- `validateReservedNameCollisions`, `findOutbound`, and upstream graph checks
  reject duplicate, ambiguous, recursive, or unsafe outbound structures.
- `validateHomeProxy` rejects placeholder endpoints or credentials, invalid
  ports, disabled UDP, and forbidden recursive targets before injection.
- `resolveUpstreamName`, `findOutbound`, `hardenReachableUpstreamGraph`, and
  `validateTopLevelUpstream` ensure the selected upstream exists, is
  unambiguous, remains acyclic, and has a usable node source.
- `parseHomeProxyToml` is intentionally a constrained parser for one
  `[home_proxy]` table. It rejects unknown/duplicate fields and unsupported value
  forms with line-numbered errors.
- `validateHomeProxyConfig` requires all eight keys and exact string, integer,
  boolean, reserved-name, and port-range contracts before output is written.

Errors in source and CLI code are Chinese and name the failed invariant. Safety
failures throw; they do not fall back to `DIRECT`, guess credentials, or emit a
partially valid local script.

## Testable Contracts

The root extension exposes selected builders and constants only inside a guarded
`module.exports` block. Tests import those contracts and use
`node:assert/strict`; they do not reimplement the production rule builder.
`scripts/sync-local-config.js` likewise exports parser, validator, renderer, and
sync functions while keeping CLI execution guarded.

## Anti-Patterns

- Do not assume `config.proxies`, `config.rules`, nested DNS fields, or outbound
  entries have the expected shape without guards.
- Do not add unchecked coercion such as converting an invalid port with
  `Number(...)` at the validation boundary.
- Do not broaden the local TOML grammar casually; every accepted form needs
  parser and rejection coverage in `tests/sync-local-config.test.js`.
- Do not introduce TypeScript-only syntax, ESM exports, or a transpilation
  requirement into the pasteable extension.
