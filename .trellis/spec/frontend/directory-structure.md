# Directory Structure

## Actual Layout

```text
clash-verge-ai-residential.js              # Clash Verge global extension and public template
clash-verge-ai-residential.local.toml.example
scripts/
  sync-local-config.js                     # strict TOML-to-local-script renderer
  check-template-safety.js                 # public-template and secret scan
tests/
  regression.test.js                       # extension behavior and routing boundary
  sync-local-config.test.js                # renderer validation and output behavior
  check-template-safety.test.js            # public-template and secret-scan behavior
docs/                                      # default-locale Chinese docs; VitePress project root
docs/en/                                   # English docs-site tree
docs/adr/                                  # frozen ADRs; not part of the VitePress site
docs/package.json                          # VitePress-only Node 22+ toolchain
package.json                               # zero-dependency Node 18+ command surface
justfile                                   # ci, render-local, and docs-dev/docs-build recipes

The ignored `clash-verge-ai-residential.local.toml` and generated
`clash-verge-ai-residential.local.js` are user-local artifacts, not source files.
The repository deliberately has no `src/`, component tree, assets directory, or
build output.

## Placement Rules

- Keep host-executed routing, DNS, migration, and configuration transformation
  logic in `clash-verge-ai-residential.js`; users paste this one file into Clash
  Verge Rev.
- Keep Node-only filesystem or CLI behavior in `scripts/`. The root extension
  must not require `node:fs`, `node:path`, or another Node-only module.
- Put extension regressions in `tests/regression.test.js`. Put renderer/parser
  regressions in `tests/sync-local-config.test.js`. Put repository secret-scan
  regressions in `tests/check-template-safety.test.js`.
- Put user-facing explanations in the matching Chinese file under `docs/` and
  keep `README.md` as the overview. English pages belong in `docs/en/` with the
  same kebab-case names. Do not rename or split the ten existing `docs/*.md`
  and `docs/agents/*.md` paths.
- Keep VitePress in `docs/package.json` and `docs/.vitepress/`. Root
  `package.json` stays zero-dependency. `just docs-dev` / `just docs-build`
  require Node.js 22+ and are not part of `just ci`.
- Leave `docs/adr/` path, filename, and body unchanged. Do not copy ADR files
  into `docs/en/`. VitePress must exclude them from the site.
- Keep public configuration shape in
  `clash-verge-ai-residential.local.toml.example`; real values belong only in
  ignored local files.

## Organization Inside The Extension

The root script uses numbered sections: identifiers and user policy constants,
domain data, helpers, upstream resolution, validation, managed rules, DNS,
configuration operations, and finally `main`. Add behavior next to the existing
section it extends. For example, domain lists such as `CORE_EXACT_DOMAINS` feed
builders such as `buildDomainRules`, while `main` only sequences the stages.

```js
const existingRules = cleanAndMigrateExistingRules(config.rules);
config.rules = dedupeRuleEntries([
  ...buildInjectedRules(),
  ...existingRules
]);
```

Use descriptive camelCase for functions and local variables, UPPER_SNAKE_CASE
for policy switches and constant tables, and kebab-case for documentation file
names. CommonJS scripts and tests use `.js`; do not introduce a parallel `.ts`
or ESM tree without an explicit repository-wide decision.

## Anti-Patterns

- Do not create fictional `components/`, `hooks/`, or `state/` layers for this
  non-UI project.
- Do not split the runtime into required imports; Clash Verge consumes the
  standalone root file.
- Do not hand-edit generated `.local.js` output or commit local credentials.
- Do not add VitePress or other docs-site packages to the root `package.json`.
- Do not fold `docs-build` into `npm run ci` or `just ci`; those gates must
  still run on Node 18 without installing `docs/node_modules`.
- Do not duplicate routing constants in scripts or tests. Import guarded test
  exports from the root extension, as `tests/regression.test.js` does.
