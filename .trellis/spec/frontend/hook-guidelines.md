# Entry Point And Side-Effect Guidelines

React-style hooks and client-side data fetching do not exist in this project.
There are two explicit execution boundaries: the Clash host entry point and the
Node local-renderer CLI.

## Clash Host Entry Point

Clash Verge Rev calls `main(config, profileName)` from
`clash-verge-ai-residential.js`. Keep it synchronous and deterministic apart
from guarded logging. It validates reserved names and the upstream graph before
injecting the proxy, rules, DNS, and sniffer configuration. It reads TUN and
IPv6 settings only to emit warnings.

```js
function warn(message) {
  if (typeof console !== "undefined" && typeof console.warn === "function") {
    console.warn(message);
  }
}
```

The extension performs no network requests, timers, subscriptions, caching, or
background work. Clash/Mihomo consumes the returned configuration and owns the
actual networking lifecycle.

### Host execution contract (verified against clash-verge-rev dev, 2026-08)

- The app re-runs the global script on every config regeneration. When a profile
  has no dedicated script item, the same global script runs a second time as the
  profile item with the already-transformed config — keep `main` idempotent.
- An exception in `main` makes the app discard ALL script changes and continue
  with the pre-script config (error surfaces in the Script card logs and the
  config log stream). This is routing fail-open: AI requests may use the
  original Profile instead of the residential chain. Treat any script error as
  a routing incident, inspect the logs, and stop AI traffic until validation
  succeeds; do not rely on throwing as a fail-closed enforcement mechanism.
- Settings-owned keys are snapshotted before and restored after script
  execution. The set is: top-level control-plane keys (`external-controller*`,
  `secret`, `mixed-port`, `socks-port`, `port`, `redir-port`, `tproxy-port`,
  `mode`, `allow-lan`, `log-level`, `ipv6`, `unified-delay`); `tun.enable`
  and the saved TUN GUI keys (`stack`, `device`, `auto-route`,
  `route-exclude-address`, `auto-redirect` on Linux, `auto-detect-interface`,
  `dns-hijack`, `strict-route`, `mtu`); and, when the DNS override is enabled,
  every non-empty `dns.*` field in `dns_config.yaml` plus a non-empty `hosts`.
- From Clash Verge Rev v2.5.5, a script change to any settings-owned key shows
  the notice "Extensions wrote <keys>, which Settings manages, so those values
  were discarded" (deduplicated by key set). The script must not write `tun`
  keys or the top-level `ipv6`. `checkHostOwnedFields` only reads these
  fields and emits at most three conditional `warn` lines (missing
  `any:53` / `tcp://any:53` in `dns-hijack`, `strict-route` off when
  `ENABLE_TUN_STRICT_ROUTE`, `ipv6 === true`) that point users at the settings
  page. Rebuilt `dns` fields survive only while the DNS override is off.
- Engine is boa_engine 0.21: no network/file IO, 5s timeout, 10M loop-iteration
  limit, 1000-line/1MB console cap, 10MB config JSON cap. `profileName` is the
  profile display name (Chinese passes through verbatim).
- Build one outbound name index per `main` and require it in `findOutbound`.
  Do not scan `proxies` / `proxy-groups` once per reachable leaf. Do not emit
  one `console.warn` per subscription node: the 1000-line cap throws, `main`
  aborts, and the host discards every script change. Aggregate or cap host
  logs. Index keys must match `namedItems` (`item` truthy, `item.name` as-is).

## Dual-Client Host Contract (2026-11)

### Scope / Trigger

Use this contract for Windows and Ubuntu entry-point compatibility. Source
baselines are Verge 2.5.8/2.5.10 and FlClash 0.9.2; installed versions and
effective runtime output still require separate verification. See the versioned
source links in `docs/configuration.md` and the host-compat task research.

### Signatures

- Verge calls synchronous `main(config, profileName)` through Boa.
- FlClash's Rust/QuickJS bridge calls `main(config)` for ordinary subscription
  or file Profiles configured with a selected Script. Its Custom Profile
  builder does not enter that script branch in the verified version.
- Neither host is Node/CommonJS. `module`, `require`, `process`, and even
  logging globals must not be required to evaluate the pasteable script.

### Contracts

Keep one business script. Missing `profileName` skips name-specific upstream
overrides and uses the existing general candidates/final-rule fallback; never
invent a Profile name. Preserve input, idempotence, managed ownership and the
current AI-only rules/defaults. FlClash applies app fields, selected DNS
overrides and optional system DNS after script evaluation. A selected
`nameserver-policy` override replaces that table. App settings also determine
`find-process-mode`; process lookup and process routing remain separate.

### Validation / Error Matrix

| Case | Required behavior or evidence |
| --- | --- |
| One or two arguments, no CommonJS globals | Same business output when no name override applies |
| Named override with omitted name | Use general fallback, not the named upstream |
| Top-level `rule-providers: null` | Normalize only the working copy; existing ownership guards still apply |
| Invalid provider structure | Reject without mutating input |
| App overwrites DNS / lookup / TUN | Inspect final config; do not infer success from the script log |
| FlClashCore executable | Do not use standalone Mihomo `-v`, `-h` or `-t`; its first argument is an IPC address |

### Good / Base / Bad Cases

Base: a normal Profile selects the script and the output preserves its unknown
rules. Good: regenerate twice, compare selected upstream and DNS, and then
verify the host's final configuration. Bad: claim profile-name overrides work
in FlClash or treat its YAML-unmarshal validation RPC as full Mihomo `-t`.

### Tests Required

The regression suite exercises the complete source in a VM without CommonJS,
process or console, both argument shapes, unchanged input, idempotence,
rejection and named/general upstream selection. These tests use V8 and do not
prove native Boa/QuickJS execution. Keep native-core, generated-file and actual
traffic evidence separate; public probes or zero ICE candidates are not proof
of correct DNS/WebRTC paths.

### Wrong vs Correct

Wrong: both clients embed Mihomo, so one Node test proves both desktop apps.
Correct: verify shared JS behavior, versioned host transforms, effective
configuration and actual traffic as distinct layers for each OS/client pair.

## Node CLI Entry Point

`scripts/sync-local-config.js` separates reusable functions from process side
effects. Tests can require its exports without running the CLI:

```js
if (require.main === module) {
  try {
    main();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
```

Keep filesystem reads and writes inside the renderer boundary. Validate the
complete local configuration before writing, reject output paths that overwrite
the template or TOML input, and retain `writeFileAtomically`'s temporary-file
cleanup. The command reports failures in Chinese and sets a nonzero exit code.

## Local TOML Switch Wiring Checklist

Adding or changing a `[routing]` / `[runtime]` switch is a cross-file contract.
`tests/sync-local-config.test.js` ("生产映射覆盖全部用户布尔开关…") enforces
every item, so a missed wire fails CI rather than shipping silently. Complete
all of them in one change:

1. Boolean constant in the public template's user-config section
   (`const ROUTE_X = true;` — the exact-anchor regex depends on this format).
2. `activeSuffixDomains()` / `activeExactDomains()` / `activeDomainRegexes()`
   gating in the template (switch-on path: rules + DNS policy).
3. The matching `allPossible*Domains()` / `allPossibleDomainRegexes()` entry
   (switch-off path: managed-rule cleanup after the switch was ever enabled).
   If the new switch receives hosts or regexes previously gated by another
   switch, keep those entries in `allPossible*()` under the new catalog. Do
   not delete them from the cleanup set because the new default is `false`.
4. The constant and domain lists in the template's `module.exports.constants`.
5. `SWITCH_CONFIG_FIELDS` entry in `scripts/sync-local-config.js`
   (`table` / `key` / `constant` / `type`).
6. The key + default in `clash-verge-ai-residential.local.toml.example`
   (single source of the auto-completion defaults).
7. The table row in both `docs/configuration.md` and
   `docs/local-configuration.md` (`| \`table.key\` | \`CONSTANT\` | \`default\` | …`).

## Local TOML Auto-Completion Contract

`completeLocalToml(localSource, localConfig, exampleConfig)` in
`scripts/sync-local-config.js` appends only missing switch keys (a whole missing
table gets `[table]` rebuilt at EOF). Invariants:

- Never rewrite existing lines: user values, comments, blank lines, BOM,
  trailing-newline presence, and dominant EOL (CRLF vs LF) are preserved
  verbatim; insert with line-splice, never parse→re-serialize.
- Idempotent: with nothing missing it returns `null` and the file is not
  touched (compare content, not mtime, in tests).
- Defaults come from the example file only; `validateExampleSwitchDefaults`
  fails the sync if the example is missing a declared key.
- `[home_proxy]` keys are never auto-completed (credentials must be hand-filled;
  `validateHomeProxyConfig` still fails closed).
- After completion, re-parse and re-validate the completed text before
  rendering; the completion write is atomic via `writeFileAtomically`.

> **Warning — same-index insertion order**: when an existing table's block tail
> coincides with EOF, key-append and whole-table-append share one insertion
> index. Whole-table splices must run first (occupying the tail), then key
> appends land before the new `[table]` header. Reversing this silently moves
> the appended keys inside the new table and the re-parse rejects them.

## Test Loading

`tests/regression.test.js` loads the public extension through CommonJS and allows
`CLASH_SCRIPT_PATH` to target a generated local script. It temporarily suppresses
`console.info` and `console.warn` in `quietMain`, restoring both in `finally`.
Use the same cleanup discipline for any temporary process state.

## Anti-Patterns

- Do not introduce React hooks, event buses, observers, or async fetch layers.
- Do not execute the local renderer on `require`; preserve the
  `require.main === module` guard.
- Do not make the Clash entry point depend on `process`, filesystem APIs, module
  loading, or persistent background state.
- Do not swallow configuration errors or leave partial output after a failed
  local render.
