# v5.5 Routing, TOML Configuration, and Usage Docs

## Goal

Ship a coherent v5.5.0 release that keeps the residential link limited to the
requested core AI traffic by default, makes every scalar switch in the script's
user-configuration section controllable from the ignored local TOML, and gives
users an unambiguous path from configuration to Clash Verge Rev's Global Extend
Script with or without `just`.

The public template must remain credential-free and directly pasteable. Local
rendering must remain one-way and must never modify the public template.

## Background And Confirmed Facts

- The current public script is v5.4.0 and injects 55 unique rules. Cursor core is
  enabled by default and contributes 12 of those rules.
- The repository's first public routing implementation (`0dd8e1d`) was already
  v5.4. The `LEGACY_*` migration paths protect unreleased/pre-repository shapes,
  not a released version in this repository.
- `scripts/sync-local-config.js` currently accepts only `[home_proxy]`, only
  renders `HOME_PROXY_TEMPLATE`, rejects unknown keys, and writes atomically.
- Existing local TOML files containing only `[home_proxy]` are already in use
  and must continue to render after the upgrade.
- `HOME_PROXY_NAME` and the exact multiline shape of `HOME_PROXY_TEMPLATE` are
  regex contracts consumed by the renderer and template-safety check.
- The current worktree already contains user changes in `.gitignore`,
  `README.md`, `docs/local-configuration.md`, and `justfile`. Implementation
  must build on those changes and must not reset or rewrite them wholesale.
- `assets/PixPin_2026-07-23_14-35-58.png` is a sanitized 2554x1525 screenshot
  that highlights Profiles -> Global Extend Script in Clash Verge Rev.

## Requirements

### R1. Narrow And Simplify The v5.5 Default Route

1. Set `ROUTE_CURSOR_CORE` to `false` in the public template. Keep the narrow
   Cursor domain catalogs so a local user can opt in.
2. Remove the three redundant Cursor catalog entries:
   - exact `repo42.cursor.sh`, covered by the bounded repository regex;
   - regex `^[a-z0-9-]+\.api5\.cursor\.sh$`, covered by the `api5.cursor.sh`
     suffix rule;
   - regional `gcpp.cursor.sh` regex, covered by that suffix rule.
3. Remove the v5.3-and-earlier migration system from the root script and its
   tests: legacy groups/catalogs/templates, target migration, group-reference
   migration, and legacy-group removal.
4. Retain a current-version managed-rule cleanup function. It must remove every
   rule this version can generate, including a rule from a switch that has since
   been disabled, while preserving unknown user-authored rules even when they
   target `AI-家宽`.
5. Deliberately treat the three removed v5.4 Cursor rule strings as non-managed
   input rather than retaining a hidden cleanup-only migration list. If a user
   manually persisted v5.4 generated output in a subscription or Merge layer,
   these exact rules therefore survive and require documented manual removal:
   - `DOMAIN,repo42.cursor.sh,AI-家宽`;
   - `DOMAIN-REGEX,^[a-z0-9-]+\.api5\.cursor\.sh$,AI-家宽`;
   - `DOMAIN-REGEX,^(?:us-asia|us-eu|us-only)\.gcpp\.cursor\.sh$,AI-家宽`.
6. Remove the duplicate realtime-suffix insertion inside the managed-rule set.
7. Reject an upstream name containing `#` or `&` before interpolating it into a
   Mihomo DoH URL and include the offending name in the Chinese error message.
8. Keep strict DNS rebuilding, `ipv6 = false`, sniffer behavior, TUN hardening,
   domain catalogs other than the three Cursor redundancies, and route order
   unchanged.
9. Do not add any domain or broaden any suffix/process match.

### R2. Expose Scalar User Switches Through Local TOML

1. Extend the constrained TOML format with optional `[routing]` and `[runtime]`
   tables. `[home_proxy]` remains required and unchanged.
2. Map all 21 scalar booleans in the root script's user-configuration section,
   not only Cursor. The exact schema and JS mapping are defined in `design.md`.
3. The tracked example must show every supported switch and mirror public
   defaults, including `routing.cursor_core = false`.
4. Both new tables are optional and may contain partial overrides. A legacy
   home-proxy-only TOML therefore inherits the public template defaults.
5. Reject unknown tables, unknown keys, duplicate tables/keys, and non-boolean
   values with line-numbered Chinese errors. Keep the supported TOML subset
   dependency-free.
6. Use one declarative mapping as the allowlist, type contract, and JS constant
   injection source so parser and renderer fields cannot drift independently.
7. Each configured switch must replace exactly one `const NAME = true|false;`
   declaration. Missing or duplicate template anchors fail before output is
   written.
8. Preserve atomic writes, input/output overwrite checks, `require.main`
   isolation, the generated-file banner, and proof that the public template is
   byte-for-byte unchanged.
9. Export or otherwise expose the declarative mapping to tests. A bidirectional
   cross-layer assertion must first prove that its JS constant names exactly
   equal the boolean declarations in the root script's user-configuration
   section, then prove that the tracked TOML example and both switch documents
   contain every mapped TOML key, JS constant, and default. The test must not
   introduce a second hard-coded switch allowlist.

### R3. Prove Default And Opt-In Behavior

1. Default public-script tests must prove Cursor AI, Marketplace, downloads,
   website/docs/forum, YouTube, shared Google resources, and telemetry do not
   route through `AI-家宽`.
2. Catalog tests must prove the retained Cursor suffix/exact/regex entries are
   narrow and still cover representative repository-number hosts.
3. Renderer tests must generate a temporary local script with
   `routing.cursor_core = true`, load that generated script through the existing
   `CLASH_SCRIPT_PATH` boundary or an equivalent isolated process, and prove
   Cursor core routing is restored without routing adjacent non-AI hosts.
4. Renderer tests must cover partial/legacy TOML compatibility, boolean
   injection, invalid type, unknown key/table, duplicate key/table, and template
   anchor failure.
5. Idempotence tests must reflect the new ownership rule: current managed rules
   are replaced, while removed legacy/non-managed rules are user-owned and are
   no longer silently deleted. The fixture must include all three retired v5.4
   Cursor rule strings and assert that their survival is intentional.
6. Rule-count tests must derive the Cursor opt-in delta from the lengths of the
   final suffix/exact/regex catalogs rather than hard-code `43`, `9`, `12`, or
   another release-specific total. Exact counts may be printed as verification
   evidence, but observable routing and uniqueness are the regression contract.

### R4. Complete Release And User Documentation

1. Synchronize v5.5.0 in `SCRIPT_VERSION`, `package.json`, README, regression
   output, and `CHANGELOG.md`.
2. Update route/configuration/DNS/troubleshooting documentation for Cursor's
   default-off behavior, TOML switches, removed automatic legacy cleanup, the
   first-query non-AI DNS latency trade-off, and `#`/`&` upstream-name rejection.
   Preserve and explain the known split-login trade-off: `auth.openai.com` and
   `accounts.google.com` stay on the original Profile while model/chat traffic
   uses the residential exit, which can trigger extra verification in strict
   risk-control scenarios.
3. Replace `docs/configuration.md`'s incomplete Scope switches block with an
   exhaustive TOML-to-JS/default table aligned with the declarative mapping. It
   must explicitly include the three switches omitted in v5.4 documentation:
   `ROUTE_GEMINI_WEB_CORE`, `ROUTE_CURSOR_CORE`, and
   `ROUTE_CLAUDE_CODE_AUXILIARY`.
4. `docs/local-configuration.md` owns the detailed switch table. Every row must
   show the TOML key, JS constant, default, effect, and any dependency so users
   do not need to infer mixed `ROUTE_*` / `ENABLE_*` naming.
5. Rename the screenshot to
   `assets/clash-verge-rev-global-extend-script.png` without changing its image
   content. Embed the same asset in README and `docs/local-configuration.md`
   with path-correct links and useful alt text.
6. README must show the complete preferred flow:
   `just render-local` -> edit ignored TOML -> rerun -> paste generated local JS
   into the screenshot's Global Extend Script -> refresh the Profile.
7. README must state that `just` is optional. Without it, document the manual
   equivalent: create/copy the ignored TOML, edit proxy values and switches,
   run `node scripts/sync-local-config.js`, then paste the generated local JS.
8. README remains concise and links to the detailed local configuration guide.
   Additions follow each target file's existing language instead of translating
   unrelated documentation.
9. Do not publish release-specific rule/switch/test totals as stable product
   promises in README or docs. Record actual counts in verification output, and
   use mapping/catalog-derived assertions in executable tests.

## Acceptance Criteria

- [ ] AC1: The public v5.5 script reports `ROUTE_CURSOR_CORE === false`, emits
  unique default rules, and emits no Cursor route or Cursor DNS policy entry.
  The targeted release audit records the currently expected observation of 43
  unique rules without making that literal the behavioral regression contract.
- [ ] AC2: Enabling only `routing.cursor_core` in a temporary local TOML emits
  the retained narrow Cursor rules and DNS policy while all documented Cursor
  non-AI hosts remain excluded. The rule-count delta equals the total derived
  from the final Cursor catalogs.
- [ ] AC3: The example TOML contains every declaratively mapped scalar switch
  (21 at planning time) with defaults equal to the public template; a
  `[home_proxy]`-only TOML still renders successfully. The production mapping's
  JS names exactly equal the boolean declarations in the script's explicit
  user-configuration section.
- [ ] AC4: Invalid tables, keys, duplicates, boolean types, and JS anchors fail
  clearly and leave neither the public template nor a partial output modified.
- [ ] AC5: Current managed rules are replaced across switch changes; unknown
  user rules survive; running `main` twice produces no duplicate proxy, group,
  rule, DNS, TUN, or sniffer entries.
- [ ] AC6: No `LEGACY_*`, `migrateLegacy*`, or v5.3 cleanup symbol remains in
  the root script or executable tests. Upgrade docs explicitly assign broad old
  rules and the three exact retired v5.4 Cursor rules to manual cleanup, while
  a regression proves those strings remain user-owned when present in input.
- [ ] AC7: `buildUpstreamDoh` accepts existing emoji/space group names and
  rejects names containing `#` or `&` before URL construction.
- [ ] AC8: `HOME_PROXY_NAME` and `HOME_PROXY_TEMPLATE` retain their renderer and
  safety-check text contracts; public credential placeholders remain safe.
- [ ] AC9: README and local-configuration docs render the renamed screenshot,
  identify Global Extend Script, and cover preferred and no-`just` Node-renderer
  flows without instructing users to hand-edit generated files or commit
  secrets.
- [ ] AC10: Version/changelog/routing/configuration/troubleshooting/DNS text is
  consistent with v5.5 defaults and contains no claim that Cursor is enabled by
  default or that legacy rules are still auto-migrated. It records the unchanged
  login-versus-model exit-IP trade-off without adding either shared auth domain.
  Mapping-driven checks prove `docs/configuration.md` and
  `docs/local-configuration.md` enumerate every TOML key, JS constant, and
  default, including the three switches missing from v5.4 docs.
- [ ] AC11: `npm run ci` passes. When available, `just ci` produces the same
  result. A sanitized real Profile check is recorded if performed; if it cannot
  be performed, it remains an explicit evidence gap rather than a pass.
- [ ] AC12: Final diff inspection shows only task-scoped additions layered onto
  pre-existing worktree changes; no generated `.local.js`, real local TOML,
  credentials, unredacted logs, commit, or push is included.

## Key Decisions

- Cursor remains supported but is opt-in and defaults to off in both the public
  template and tracked TOML example.
- All scalar boolean switches in the explicit user-configuration section share
  one TOML override mechanism. Structured arrays/maps and domain/DNS catalogs do
  not, because they require a materially larger TOML grammar and validation
  contract.
- Existing home-proxy-only local TOML files remain valid through optional,
  partial switch tables.
- Automatic unreleased-version migration is removed; current managed-rule
  replacement and idempotence remain mandatory.
- Retired v5.4 Cursor rule strings are not smuggled back into a cleanup-only
  migration set. Normal Profile regeneration drops them; manually persisted
  copies are preserved as user-owned input and covered by exact removal docs.
- No new dependency is introduced; regex-based template injection is retained
  with exact-one-anchor guards.
- Planning completion does not authorize implementation. `task.py start` waits
  for explicit approval of these final artifacts in a later user message.

## Out Of Scope

- Adding, refreshing, or broadening AI domain catalogs.
- Changing strict DNS construction, IPv6, sniffer, TUN, fake-IP, or route-order
  policy beyond the upstream-name delimiter validation.
- Exposing `PROFILE_UPSTREAM_OVERRIDES`, `UPSTREAM_CANDIDATES`, domain arrays,
  DNS endpoint arrays, names, ports, or other structured constants in TOML.
- Adding a general TOML package, build step, UI, or reverse sync from generated
  JavaScript back into TOML.
- Automatically deleting the three retired v5.4 Cursor strings from a user's
  manually persisted subscription/Merge rules.
- Cropping, annotating, recompressing, or duplicating the supplied screenshot.
- Committing, pushing, opening a PR, or modifying ignored real local files.
