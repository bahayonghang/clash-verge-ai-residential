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

### Reachable upstream empty fallback

`hardenReachableUpstreamGraph(config, upstreamName, outboundIndex)` rejects
`empty-fallback` equal to `家宽-SOCKS5` or `AI-家宽` on each visited group.
Check after adding the group to the DFS path and before editing that group.
An include-all exclusion does not remove an explicit fallback reference.
The error must name the group, field and reachable path without credentials;
`main` must leave its input unchanged. Never delete the field or substitute
DIRECT/COMPATIBLE. Ordinary proxy fallbacks, absent fields and unreachable
groups retain their behavior. Test both top-level and nested rejection,
ordinary fallback preservation, unchanged input and repeated execution.
Host fallback after a script error remains outside this validation contract.

## Scenario: Regex residential DNS

### 1. Scope / Trigger

Existing Vertex and Cursor indexing regexes need residential DNS without wider
Google or Cursor suffixes. Reuse `activeDomainRegexes()` and its existing gates.

### 2. Signatures

`main(config, profileName)` returns a cloned configuration with a reserved
`rule-providers["AI-家宽-DNS-REGEX"]` and
`dns["nameserver-policy"]["rule-set:AI-家宽-DNS-REGEX"]` when regexes are active.
`buildNameserverPolicy(existingPolicy)` exposes the policy keys for tests;
validate provider references through the complete `main` output.

### 3. Contracts

The provider is local `type: inline`, `behavior: classical`. Its payload contains
only `DOMAIN-REGEX,<pattern>` entries derived from active patterns. The DNS
policy uses `RESIDENTIAL_DOH`. No remote URL, subscription, new switch or business
rule is added. Insert the policy before broad geosite entries. Keep private,
bootstrap, non-AI and extra-site DNS exemptions unchanged.

Recognize previous managed providers from the full known pattern catalog,
independent of current switches. Require the canonical fields and a unique,
nonempty known payload. Clone the provider map before changes; preserve unknown
providers. When all regexes are off, remove the managed provider and policy.
Do not leave an empty provider map solely for managed output.

### 4. Validation & Error Matrix

A non-object provider map, a noncanonical same-name provider, or a user rule/other
policy reference to the reserved provider must fail without changing input.
Check nested logical rules, sub-rules, multi-provider policy keys,
`dns.fake-ip-filter`, `sniffer.skip-domain` and `sniffer.force-domain` for the
reserved name. The canonical managed DNS policy reference is allowed on rerun.
Do not delete a provider while leaving a user reference unresolved.

### 5. Good / Base / Bad Cases

Base: Vertex on and indexing off produce only the Vertex pattern. Good: each
switch controls its own pattern through on/off/on and repeated `main` calls.
Bad: use `+.googleapis.com`, `+.cursor.sh`, a raw regex key, or a wildcard inside
a DNS label as a substitute for the provider.

### 6. Tests Required

Test four switch combinations, positive and negative hosts, canonical provider
ownership, user-reference rejection, stale policy cleanup with unmanaged-policy
preservation on/off, unchanged input and renderer output. Preserve the historical
v5.11 fixture; declare only the approved DNS delta explicitly. Test a real
Mihomo process with fresh DNS queries and observed proxy hops. Loopback fixtures
prove matching and chain selection, not public DoH TLS or a supplier's egress.

### 7. Wrong vs Correct

Wrong: infer real DNS routing from a fake-IP response or generated object alone.
Correct: record the tested core version and actual resolver/proxy observations;
keep untested production DNS/UDP behavior explicit.

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
