# State Management

There is no state library, browser store, server cache, or persistent runtime
session. State is the Clash configuration object passed into the extension plus
module-level policy constants that describe the transformation.

## State Categories

| State | Owner | Local pattern |
|---|---|---|
| Input/output configuration | Clash Verge Rev | `main(config, profileName)` mutates and returns `config` |
| Policy switches and domain tables | Root extension module | `ROUTE_*`, `ENABLE_*`, and domain arrays are read during each run |
| Derived rules, DNS, and groups | Builder functions | Recomputed from current input and policy on every invocation |
| Local credentials | Ignored TOML file | Read only by `scripts/sync-local-config.js` |
| Test state | Individual test process | Fixtures create fresh objects and restore temporary mutations in `finally` |

## Ownership And Updates

Normalize missing top-level arrays at the start of `main`, validate before
overwriting reserved names, and replace managed sections through builders:

```js
if (!Array.isArray(config.proxies)) config.proxies = [];
if (!Array.isArray(config["proxy-groups"])) config["proxy-groups"] = [];
if (!Array.isArray(config.rules)) config.rules = [];
```

Nested builders use copies when merging user input. `buildDnsConfig` starts from
`cloneObject(existingDns)`, removes incompatible paths, and returns a new DNS
object. `upsertNamedItem` starts with `items.slice()` and guarantees a single
managed named item.

Treat script-managed and user-managed state differently. Functions such as
`cleanAndMigrateExistingRules` remove known current/legacy managed rules but
preserve unknown custom rules. Never replace an ambiguous or unexpected
same-name object silently; validation must fail instead.

## Idempotence

Running `main` twice on the same object must not add duplicate proxies, groups,
rules, filters, or DNS entries. `uniqueStrings`, `uniqueScalars`,
`dedupeRuleEntries`, migration sets, and upsert helpers enforce this contract.
`tests/regression.test.js` contains the authoritative repeated-execution test.

The renderer is one-way state flow:

```text
public template + ignored local TOML -> ignored generated local script
```

It never writes credentials back to the public template or TOML input.

## Anti-Patterns

- Do not add mutable singleton state, caches, or cross-run accumulators.
- Do not derive configuration once at module load when it depends on the current
  profile or input config.
- Do not append managed entries without deduplication and migration handling.
- Do not preserve unknown DNS policy paths when the strict mode deliberately
  removes alternate resolution routes.
