# Technical Design

## Boundaries

This release keeps three execution surfaces separate:

1. `clash-verge-ai-residential.js` is the public, dependency-free Clash Verge
   Rev extension and owns runtime routing behavior and public defaults.
2. `scripts/sync-local-config.js` is a Node.js 18+ local compiler from a safe
   public template plus ignored TOML to ignored local JavaScript.
3. README/docs explain the two configuration workflows and the exact paste
   target. They do not become another source of runtime defaults.

The data flow remains one-way:

```text
public JS defaults + ignored local TOML
  -> constrained parser and validation
  -> exact guarded constant/template replacement
  -> atomic ignored .local.js
  -> Clash Verge Rev Profiles / Global Extend Script
```

## TOML Contract

`[home_proxy]` remains required. `[routing]` and `[runtime]` are optional and
partial; omitted keys retain the current public-JS value.

### Routing Switch Mapping

| TOML key | JavaScript constant | v5.5 default |
| --- | --- | --- |
| `routing.openai_shared_dependencies` | `ROUTE_OPENAI_SHARED_DEPENDENCIES` | `false` |
| `routing.claude_shared_dependencies` | `ROUTE_CLAUDE_SHARED_DEPENDENCIES` | `false` |
| `routing.antigravity_google_auth` | `ROUTE_ANTIGRAVITY_GOOGLE_AUTH` | `false` |
| `routing.antigravity_project_apis` | `ROUTE_ANTIGRAVITY_PROJECT_APIS` | `false` |
| `routing.antigravity_update_and_telemetry` | `ROUTE_ANTIGRAVITY_UPDATE_AND_TELEMETRY` | `false` |
| `routing.gemini_web_core` | `ROUTE_GEMINI_WEB_CORE` | `true` |
| `routing.cursor_core` | `ROUTE_CURSOR_CORE` | `false` |
| `routing.cursor_process_fallback` | `ROUTE_CURSOR_PROCESS_FALLBACK` | `false` |
| `routing.claude_code_auxiliary` | `ROUTE_CLAUDE_CODE_AUXILIARY` | `false` |
| `routing.ai_process_fallback` | `ENABLE_AI_PROCESS_FALLBACK` | `false` |
| `routing.anthropic_ip_fallback` | `ENABLE_ANTHROPIC_IP_FALLBACK` | `true` |
| `routing.shared_realtime_infrastructure` | `ROUTE_SHARED_REALTIME_INFRASTRUCTURE` | `false` |
| `routing.global_realtime_ports` | `ROUTE_GLOBAL_REALTIME_PORTS` | `false` |
| `routing.public_encrypted_dns` | `ROUTE_PUBLIC_ENCRYPTED_DNS` | `false` |

Dependencies must be documented, not silently normalized:

- `cursor_process_fallback` is effective only when `ai_process_fallback` is on.
- `global_realtime_ports` is effective only when
  `shared_realtime_infrastructure` is on.

### Runtime Switch Mapping

| TOML key | JavaScript constant | v5.5 default |
| --- | --- | --- |
| `runtime.allow_final_rule_upstream_fallback` | `ALLOW_FINAL_RULE_UPSTREAM_FALLBACK` | `true` |
| `runtime.allow_heuristic_upstream_fallback` | `ALLOW_HEURISTIC_UPSTREAM_FALLBACK` | `false` |
| `runtime.preserve_unmanaged_nameserver_policy` | `PRESERVE_UNMANAGED_NAMESERVER_POLICY` | `false` |
| `runtime.enable_domain_sniffer` | `ENABLE_DOMAIN_SNIFFER` | `true` |
| `runtime.harden_existing_tun_dns_hijack` | `HARDEN_EXISTING_TUN_DNS_HIJACK` | `true` |
| `runtime.enable_tun_strict_route` | `ENABLE_TUN_STRICT_ROUTE` | `false` |
| `runtime.warn_on_reachable_udp_disabled` | `WARN_ON_REACHABLE_UDP_DISABLED` | `true` |

`enable_tun_strict_route` only has an effect when existing TUN hardening runs.
The documentation must call out that enabling permissive/fallback switches can
change privacy, cost, compatibility, or traffic-scope guarantees.

## Parser And Renderer

### Single Mapping Owner

Introduce one frozen/declarative mapping whose rows contain:

- TOML table;
- TOML key;
- JavaScript constant name;
- expected type (`boolean`).

The parser derives known tables/keys from this mapping. Validation walks the
same mapping. Injection walks the same mapping. Do not maintain parallel field
lists that can drift.

Expose the mapping to tests. The test extracts boolean `const` declarations from
the bounded user-configuration section of the public script and compares that
set bidirectionally with the mapping's JS names. It then uses the mapping rows to
check the tracked example and documentation coverage. It must not duplicate the
21 keys or JS constant names in another assertion-only array.

The existing eight `[home_proxy]` fields keep their current contract and can be
represented alongside, or composed with, the switch mapping without weakening
their specialized validation.

### Parsed Shape

Rename the narrowly named parser only if doing so improves clarity. Its returned
shape should make ownership explicit, for example:

```js
{
  homeProxy: { /* eight required values */ },
  routing: { cursor_core: true },
  runtime: {}
}
```

No table/key is accepted before its table header. Repeated table headers and
repeated assignments fail. The existing comment, quoted-string, integer, and
boolean handling remains unchanged; the new tables accept booleans only.

### Injection

Apply transformations in memory before any write:

1. validate complete parsed input;
2. replace the single `HOME_PROXY_TEMPLATE` block;
3. for each supplied switch, locate exactly one declaration matching
   `const <JS_NAME> = true|false;` and replace only the literal;
4. prepend the generated banner;
5. atomically write the result.

Constant names come only from the trusted mapping, never directly from TOML.
Escape any name used to build a regex even though current names are alphanumeric
and underscores. An absent or multiply matched declaration is a template error,
not a partially successful render.

## Runtime Routing Changes

### Cursor

The public default is off. Catalogs remain exported for focused tests and local
opt-in. The retained catalog is:

- suffixes: `api2.cursor.sh`, `api5.cursor.sh`, `gcpp.cursor.sh`,
  `authentication.cursor.sh`;
- exact: `api3.cursor.sh`, `api4.cursor.sh`, `authenticator.cursor.sh`,
  `api.cursor.com`;
- regex: `^repo[0-9]+\.cursor\.sh$`.

Suffix semantics cover subdomains, so the removed API5/GCPP regexes add no
matching capability. The repository regex covers the removed `repo42` exact
entry.

### Managed Rules Without Legacy Migration

Rename `cleanAndMigrateExistingRules` to a name that reflects its remaining
responsibility, such as `cleanExistingManagedRules`.

`buildManagedRuleSet()` continues to enumerate all rules the current script can
generate across enabled and disabled switch states, targeting only `AI-家宽`.
This is essential for changing a local TOML switch from true to false. It no
longer enumerates unpublished legacy groups, catalogs, templates, or targets.

The cleaner removes exact current-managed strings, then deduplicates everything
else without retargeting or deleting unknown rules. A broad old rule such as
`DOMAIN-SUFFIX,cursor.com,AI-家宽` is therefore user-owned after v5.5 and must be
documented for manual removal.

The same ownership rule applies to the three strings that v5.4 generated from
the now-redundant Cursor entries:

```text
DOMAIN,repo42.cursor.sh,AI-家宽
DOMAIN-REGEX,^[a-z0-9-]+\.api5\.cursor\.sh$,AI-家宽
DOMAIN-REGEX,^(?:us-asia|us-eu|us-only)\.gcpp\.cursor\.sh$,AI-家宽
```

Do not retain those values in `buildManagedRuleSet()` or introduce a renamed
retired-rule cleanup constant. Doing so would reintroduce the migration system
under a different label. Normal Clash Verge regeneration never sees the prior
script output; users who copied generated rules into subscription/Merge input
must remove these exact strings using the upgrade documentation.

### DoH Upstream Name

`buildUpstreamDoh` trims the resolved name as today, rejects empty values, then
rejects `#` or `&` because Mihomo uses those characters to delimit the proxy
fragment and query parameters. Existing names with spaces or emoji remain
accepted unchanged. Do not URI-encode names without separate Mihomo evidence.

## Test Design

### Root Regression Suite

- Assert v5.5 version/defaults and observable absence of Cursor routes/DNS.
- Assert exact retained Cursor catalogs and regex coverage without re-injecting
  them in the default build.
- Preserve explicit non-AI negative cases.
- Replace legacy migration tests with current-managed cleanup and user-owned
  unknown-rule preservation tests. Include the three retired v5.4 Cursor rules
  and representative broad pre-v5.4-like rules as explicit preserved inputs.
- Add DoH delimiter rejection while retaining emoji/space success.
- Keep double-run idempotence and no-duplicate assertions.
- Compare default and Cursor-opt-in generated output using the sum of the final
  Cursor catalog lengths. Do not hard-code a total rule count in the regression.

### Local Renderer Suite

- Preserve a valid `[home_proxy]`-only fixture as the backward-compatibility
  case.
- Add a complete/partial switch fixture and assert exact generated constants.
- Extract the public script's user-configuration boolean declarations and prove
  their names exactly equal the production switch mapping, then iterate that
  mapping to prove every example-TOML entry has the same default and every
  documented table includes the TOML key plus JS constant. Do not create a
  test-only field list.
- Generate a local script with Cursor enabled and test behavior in an isolated
  Node process so CommonJS module caching cannot mask the generated constants.
- Add rejection cases for unknown table/key, duplicate table/key, wrong boolean
  type, and zero/multiple JS anchors.
- For every failed render, assert no partial output; for successful render,
  assert the public template is unchanged.

Tests should verify behavior produced by the generated script rather than only
repeating mapping literals.

## Documentation And Asset Placement

Rename the existing file in place to:

```text
assets/clash-verge-rev-global-extend-script.png
```

Reference it from:

- README: `assets/clash-verge-rev-global-extend-script.png`;
- `docs/local-configuration.md`:
  `../assets/clash-verge-rev-global-extend-script.png`.

Place the screenshot immediately beside the paste/refresh instructions. Alt
text must identify Clash Verge Rev Profiles and Global Extend Script; do not use
the original capture filename as alt text.

README presents a short preferred path plus the direct Node equivalent when
`just` is unavailable. The local configuration document owns the full TOML
tables, switch dependencies, Windows PowerShell and macOS/Linux copy commands,
renderer errors, and credential boundary. Both paths edit TOML and generate the
local script; neither instructs users to hand-edit generated output.

`docs/local-configuration.md` is the detailed user-facing switch reference and
shows TOML key, JavaScript constant, default, effect, and dependencies.
`docs/configuration.md` replaces its incomplete JavaScript code block with the
same exhaustive mapping-level information. At minimum, cross-layer checks must
catch omission of `ROUTE_GEMINI_WEB_CORE`, `ROUTE_CURSOR_CORE`, or
`ROUTE_CLAUDE_CODE_AUXILIARY`, the three gaps in v5.4 documentation.

Routing documentation also keeps the existing authentication split explicit:
shared OpenAI/Google login hosts remain on the original Profile while core model
traffic uses the residential exit. This is a documented risk-control trade-off,
not authorization to add those domains in v5.5.

## Compatibility, Risk, And Rollback

- Old local TOML remains valid because switch tables are optional.
- Old manually persisted broad rules and the three retired v5.4 Cursor strings
  are no longer removed. Changelog and troubleshooting must give exact
  search/removal guidance before users refresh.
- Regex injection remains coupled to stable public declaration shapes. Exact-one
  checks convert template drift into a clear local error instead of corruption.
- The source route changes and renderer changes roll back together. Reverting
  only one side can make the example/defaults or generated behavior disagree.
- The screenshot rename is recoverable through Git; no content transformation
  is planned.
- Real Clash/Mihomo behavior remains an integration gap unless a sanitized
  Profile check is actually performed.
