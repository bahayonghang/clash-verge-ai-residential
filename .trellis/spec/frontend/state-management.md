# State Management

There is no state library, browser store, server cache, or persistent runtime
session. State is the Clash configuration object passed into the extension plus
module-level policy constants that describe the transformation.

## State Categories

| State | Owner | Local pattern |
|---|---|---|
| Input/output configuration | Clash Verge Rev | `main(config, profileName)` transforms a clone and leaves input unchanged |
| Policy switches and domain tables | Root extension module | Public defaults are read during each run; ignored TOML can render scalar boolean overrides into a private script |
| Derived rules, DNS, and groups | Builder functions | Recomputed from current input and policy on every invocation |
| Local credentials and scalar overrides | Ignored TOML file | Read only by `scripts/sync-local-config.js` |
| Test state | Individual test process | Fixtures create fresh objects and restore temporary mutations in `finally` |

## Ownership And Updates

Clone editable sections at the start of `main`, validate before overwriting
reserved names, and replace managed sections through builders:

```js
const working = cloneConfigForEdit(config);
validateReservedNameCollisions(working);
// All subsequent transformations operate on working, not config.
```

Nested builders use copies when merging user input. `buildDnsConfig` starts from
`cloneObject(existingDns)`, removes incompatible paths, and returns a new DNS
object. `upsertNamedItem` starts with `items.slice()` and guarantees a single
managed named item.

Treat script-managed and user-managed state differently.
`cleanExistingManagedRules` removes exact rules the current version can
generate across enabled and disabled switch states, then preserves unknown
input unchanged even when its target is `AI-家宽`. Explicit retired literals in
the current `allPossible*` catalogs remain managed; older strings absent from
those catalogs remain user-owned and must be removed at their source. Never replace an ambiguous or
unexpected same-name object silently; validation must fail instead.

The three v5.4 Cursor strings removed as redundant are concrete ownership
fixtures: the current cleaner must preserve them. Current catalog output and
explicitly retained forms such as `DOMAIN-SUFFIX,api2.cursor.sh,AI-家宽` are
cleaned even when their switch is disabled.

### Core switches and reserved group ownership

`anthropic_core`, `gemini_api_core`, and `antigravity_core` default to true.
Active domain lists and exact/suffix DNS follow each switch. Dedicated process
fallbacks also require the corresponding core; Cursor additionally requires its
process switch. Anthropic CIDR fallback requires both anthropic_core and
anthropic_ip_fallback. Authentication, auxiliary and global capture switches
remain independent. Full possible-domain/process/IP cleanup never depends on
the current active switch state.

An existing `AI-家宽` group is managed only when it is a select with exactly
`家宽-SOCKS5` as its explicit member and no keys outside
`name/type/proxies/disable-udp/icon/hidden`. Extra sources, filters or alternate
selection fields reject before input mutation. `buildAiGroup` constructs the
canonical output and copies only icon/hidden, not the whole old group.
Tests cover extra-key rejection, canonical reruns, display metadata and unchanged
input. A thrown script can make the host reuse its original configuration; this
ownership check does not enforce runtime traffic blocking.

## Idempotence

Running `main` twice on the same object must not add duplicate proxies, groups,
rules, filters, or DNS entries. `uniqueStrings`, `uniqueScalars`,
`dedupeRuleEntries`, the current managed-rule set, and upsert helpers enforce
this contract. `tests/regression.test.js` contains the authoritative
repeated-execution and retired-rule ownership tests.

Managed-rule cleanup matches exact strings against `buildManagedRuleSet()`.
Consequences when evolving rule shapes:

- When a domain changes form (e.g. `api.openai.com` exact -> suffix in v5.7),
  keep the legacy literal in `allPossibleExactDomains()` so previous output is
  still cleaned; add a regression test with the legacy rule string.
- When a catalog is split onto a new switch (e.g. Cursor `repo[0-9]+` regexes
  leaving `cursor_core` in v5.9), put the new catalog in the matching
  `allPossible*()` function even if the new switch defaults to `false`. Cleanup
  enumerates every rule the current version can generate, not the active
  default. Dropping the split catalog from `allPossible*()` leaves the previous
  managed rule in the Profile. Retired exact/regex strings outside the current
  full catalogs stay user-owned.
- Never inject managed rules that embed a dynamically resolved name (such as the
  upstream group) — exact-string cleanup cannot enumerate past values and would
  either leak stale rules or force prefix matching that risks deleting
  user-owned rules. This is why `downloads.claude.ai` stays under the
  `claude.ai` suffix instead of being split to the upstream.

The renderer is one-way state flow:

```text
public defaults + ignored local credentials/switches -> ignored generated local script
```

It never writes credentials back to the public template or TOML input.

## Anti-Patterns

- Do not add mutable singleton state, caches, or cross-run accumulators.
- Do not derive configuration once at module load when it depends on the current
  profile or input config.
- Do not append managed entries without exact current-version cleanup and
  deduplication.
- Do not drop a split catalog from `allPossible*()` because the new switch
  defaults to off. The cleaner must still see that rule string.
- Do not add blanket historical migrations; preserve the explicit current catalogs.
- Do not preserve unknown DNS policy paths when the strict mode deliberately
  removes alternate resolution routes.
