# Implementation Plan

## Preconditions And Scope Guard

- Do not run `task.py start` until the user explicitly approves the latest
  planning summary in a subsequent message.
- Before editing, capture `git status --short` and the diffs for `.gitignore`,
  `README.md`, `docs/local-configuration.md`, and `justfile`. Preserve those
  user changes and layer task work on top.
- Do not edit or inspect ignored real credentials unless the user explicitly
  asks. Never add generated `.local.js` or a real `.local.toml` to Git.
- Preserve the exact `HOME_PROXY_NAME` declaration and `HOME_PROXY_TEMPLATE`
  outer text shape required by the renderer/safety scanner.

## Ordered Work

### 1. Root v5.5 Policy And Cleanup

- [ ] Update header/version and make `ROUTE_CURSOR_CORE` default false.
- [ ] Remove the three proven-redundant Cursor catalog entries.
- [ ] Keep the resulting three retired v5.4 rule strings out of the new managed
      set; do not replace deleted migration code with a cleanup-only alias.
- [ ] Remove all legacy constants, templates, migration functions, references,
      exports, and stale migration comments.
- [ ] Retain/rename current managed-rule cleanup and remove duplicate realtime
      suffix enumeration.
- [ ] Add `#`/`&` validation to `buildUpstreamDoh` without changing valid name
      output.
- [ ] Search every changed symbol before and after edits; do not refactor
      unrelated O(N) lookups or routing/DNS builders.

### 2. Local TOML Contract And Renderer

- [ ] Add one declarative 21-switch mapping for TOML tables/keys, JS constants,
      and boolean types.
- [ ] Generalize parsing to `[home_proxy]`, optional `[routing]`, and optional
      `[runtime]` while keeping the constrained grammar and line-numbered errors.
- [ ] Keep `[home_proxy]` required and validate it with the existing invariants.
- [ ] Add exact-one boolean declaration injection and compose it with the home
      proxy template injection before atomic output.
- [ ] Update exported test helpers without exposing filesystem side effects on
      `require`.
- [ ] Extend the tracked TOML example with every switch and its v5.5 default.
- [ ] Export the production mapping for mapping-driven example/default/docs
      coverage; do not add a parallel test allowlist.
- [ ] Keep `just render-local` behavior and the user's existing first-run copy
      logic; only adjust wording if needed for the expanded config.

### 3. Focused Tests

- [ ] Update root imports/version and replace legacy expectations with current
      managed-rule ownership assertions.
- [ ] Prove default Cursor route and DNS absence plus retained narrow catalogs.
- [ ] Prove valid DoH names and invalid `#`/`&` names.
- [ ] Add parser/render success and rejection cases for the two new tables.
- [ ] Prove legacy `[home_proxy]`-only TOML compatibility.
- [ ] Generate and behavior-test a Cursor-enabled temporary local script in an
      isolated process; include Marketplace/download/site negative assertions.
- [ ] Assert failed rendering leaves no partial output and every successful
      render leaves the public template unchanged.
- [ ] Keep idempotence, rule uniqueness, credential safety, and public template
      anchor tests meaningful after the migration removal.
- [ ] Put all three retired v5.4 Cursor strings plus representative broad old
      rules in the ownership fixture and assert they remain untouched.
- [ ] Derive the Cursor opt-in rule delta from final catalog lengths; record 43
      as release verification output only, not a fixed regression assertion.
- [ ] Compare user-configuration boolean declarations bidirectionally with the
      production mapping, then iterate the mapping to verify example defaults
      and exhaustive TOML-key/JS-constant coverage in both switch documents.

### 4. Release Documentation And Screenshot

- [ ] Rename the supplied image to
      `assets/clash-verge-rev-global-extend-script.png` with no pixel changes.
- [ ] Update README scope/version/defaults, preferred flow, optional `just`,
      direct Node equivalent, paste location, screenshot, and validation
      description. Do not tell users to hand-edit generated output.
- [ ] Expand `docs/local-configuration.md` with all switch tables, dependencies,
      platform commands, both renderer invocation paths, screenshot, and
      credential boundary. Each switch row includes TOML key, JS constant,
      default, effect, and dependency.
- [ ] Update `docs/configuration.md`, `docs/routing-scope.md`,
      `docs/troubleshooting.md`, and `docs/dns-and-leak-model.md` to match the
      final behavior without translating unrelated text. Preserve the known
      auth-host/model-host exit-IP split as documentation only. Replace the
      incomplete Scope switches list and explicitly include the three v5.4
      omissions.
- [ ] Put the exact three retired v5.4 Cursor rule strings in CHANGELOG and
      troubleshooting manual-cleanup guidance alongside broad old rules.
- [ ] Add v5.5.0 release notes and synchronize `package.json`, README, and test
      output. Derive any displayed test count from the final suite.
- [ ] Verify both Markdown image paths resolve on disk and no reference to the
      original `PixPin_*` filename remains.

### 5. Project Knowledge Sync

- [ ] If implementation changes the documented parser/ownership contracts,
      update the relevant `.trellis/spec/frontend/` guides during Phase 3.3.
- [ ] Keep spec edits limited to verified post-implementation behavior; do not
      rewrite general guidelines.

## Validation Sequence

Run the narrowest checks first and stop to diagnose any failure:

```powershell
node --check clash-verge-ai-residential.js
node --check scripts/sync-local-config.js
node tests/sync-local-config.test.js
node tests/regression.test.js
npm run check:secrets
npm run ci
```

Then run targeted audits:

```powershell
node -e "const m=require('./clash-verge-ai-residential.js'); const r=m.buildInjectedRules(); console.log({version:m.constants.SCRIPT_VERSION,cursor:m.constants.ROUTE_CURSOR_CORE,count:r.length,unique:new Set(r).size}); console.log(r.join('\n'))"
rg -n "LEGACY_|migrateLegacy|v5\.3.*(clean|migrat)" clash-verge-ai-residential.js tests
rg -n "PixPin_2026-07-23_14-35-58|ROUTE_CURSOR_CORE = true|Cursor.*default.*enabled" README.md docs clash-verge-ai-residential.js tests assets
git diff --check
git status --short
```

Expected targeted results:

- public summary is v5.5.0 and Cursor false; the release audit currently
  observes 43 rules/43 unique without embedding that literal in regression
  behavior or user documentation;
- no active legacy/migration symbol in source or executable tests;
- no stale screenshot filename or default-enabled Cursor claim;
- public template placeholders and generated-file exclusions remain intact.

If `just` is installed, also run `just ci` as the documented wrapper. A real
Profile check should confirm representative Claude/OpenAI/Gemini traffic hits
`AI-家宽`, Cursor does not by default, TOML opt-in restores Cursor core only,
and Marketplace/YouTube stay outside. Record it only if actually performed.

## Final Review And Rollback Points

- Inspect the complete diff by file and compare pre-existing dirty hunks so no
  user edit was lost.
- Treat route policy + TOML defaults + example + docs + tests as one rollback
  unit; do not leave mixed v5.4/v5.5 behavior.
- Confirm the image rename is the only binary change.
- Do not commit or push unless separately requested after validation.

## Execution Record - 2026-07-23

- Implemented the reviewed root-script, local renderer, example TOML, tests,
  release docs, screenshot rename, package version, and code-spec updates.
- `node tests/regression.test.js` passed all v5.5 routing regressions.
- `node tests/sync-local-config.test.js` passed the mapping-driven parser,
  renderer, isolated generated-script, documentation, and atomic-failure cases.
- `npm run ci` and `just ci` passed after the final implementation fixes.
- Targeted audit observed v5.5.0, Cursor disabled, 43 rules / 43 unique, and no
  default Cursor rule or DNS policy. This count remains evidence, not a stable
  product or regression-test promise.
- The renamed screenshot remains 2554x1525 with SHA-256
  `386F9C3DEE06EF536880D2B4AFCB048E08A434B4EBAF069D44C40AF5A4CDA4D7`.
- Evidence gap: no sanitized real Clash Verge Rev Profile/Mihomo integration
  check was performed; ignored local credentials and generated files were not
  inspected.
- No commit or push was performed, as required by this plan. Keep the task
  `in_progress` until Phase 3.4 is separately authorized and completed.
