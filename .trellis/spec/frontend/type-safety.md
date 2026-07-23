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
- `parseLocalToml` is intentionally constrained to required `[home_proxy]` plus
  optional, partial `[routing]` and `[runtime]` tables. It rejects unknown or
  duplicate tables/keys and unsupported value forms with line-numbered errors.
- `SWITCH_CONFIG_FIELDS` is the only owner of each switch's TOML table/key,
  JavaScript constant, and expected type. Supported switch tables, parser
  allowlists, validation, injection, tests, the tracked example, and switch
  documentation derive from or are checked against this mapping.
- `validateLocalConfig` composes the eight-key home-proxy contract with mapped
  boolean validation before any output is written.

Errors in source and CLI code are Chinese and name the failed invariant. Safety
failures throw; they do not fall back to `DIRECT`, guess credentials, or emit a
partially valid local script.

## Scenario: Local TOML switch compilation

### 1. Scope / Trigger

Use this contract whenever adding, renaming, or removing a scalar switch in the
root script's bounded `// 1. 用户配置` section. Structured arrays, maps, domain
catalogs, endpoints, and ports are outside this TOML surface.

### 2. Signatures

```javascript
parseLocalToml(source) -> { homeProxy, routing, runtime }
validateLocalConfig(config, homeProxyName) -> void
injectBooleanConstants(templateSource, config) -> string
syncLocalConfig({ templatePath, configPath, outputPath }) -> { configPath, outputPath }
```

### 3. Contracts

- `[home_proxy]` is required and retains its eight existing keys and value
  contracts.
- `[routing]` and `[runtime]` are optional and partial; omitted switches retain
  the public script default.
- Every `SWITCH_CONFIG_FIELDS` row has `{ table, key, constant, type }`, where
  `type` is currently `"boolean"`.
- A supplied switch replaces exactly one full-line declaration matching
  `const CONSTANT = true|false;` after home-proxy rendering and before the
  atomic write.
- Rendering is one-way: public template plus ignored TOML produces ignored
  `.local.js`; neither input is modified.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Missing `[home_proxy]` | Line-independent Chinese configuration error |
| Unknown or repeated table | Line-numbered Chinese error |
| Unknown or repeated key | Line-numbered Chinese error naming `table.key` |
| Mapped value is not boolean | Line-numbered `true` / `false` error |
| Home-proxy key/type/name/port is invalid | Existing fail-closed home-proxy error |
| Configured constant has zero or multiple template anchors | Template error naming the constant |
| Output equals template or TOML path | Reject before reading/writing output |

Every failure occurs before `writeFileAtomically`; no partial output may remain.

### 5. Good/Base/Bad Cases

- Good: `[home_proxy]` plus `routing.cursor_core = true` produces an isolated
  local script whose Cursor core rules and DNS policy are enabled.
- Base: a home-proxy-only TOML still renders and inherits every public default.
- Bad: `cursor_core = "true"`, an unknown table/key, a duplicate declaration,
  or a missing/duplicate JS anchor fails without modifying the template or
  creating output.

### 6. Tests Required

- Compare mapped constant names bidirectionally with all boolean declarations
  in the bounded user-configuration section.
- Iterate the production mapping to compare example defaults and exhaustive
  rows in both switch documents; do not create a test-only switch list.
- Exercise every parser rejection through `syncLocalConfig` and assert output
  absence plus public-template byte equality.
- Load a generated opt-in script in a separate Node process and assert both
  intended routing and adjacent non-AI exclusions.

### 7. Wrong vs Correct

#### Wrong

```javascript
const SUPPORTED_ROUTING_KEYS = ["cursor_core"];
const CONSTANT_BY_KEY = { cursor_core: "ROUTE_CURSOR_CORE" };
```

Parallel parser and renderer lists drift when a switch changes.

#### Correct

```javascript
const SWITCH_CONFIG_FIELDS = Object.freeze([
  {
    table: "routing",
    key: "cursor_core",
    constant: "ROUTE_CURSOR_CORE",
    type: "boolean"
  }
]);
```

Derive supported tables/keys and validation from the same rows used for
injection.

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
- Do not maintain a second switch table/key/type allowlist beside
  `SWITCH_CONFIG_FIELDS`.
- Do not introduce TypeScript-only syntax, ESM exports, or a transpilation
  requirement into the pasteable extension.
