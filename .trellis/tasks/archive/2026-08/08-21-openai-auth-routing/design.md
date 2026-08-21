# Technical Design

## Scope And Runtime Boundaries

This is one cohesive routing-contract change across the pasteable extension,
the local TOML compiler, tests, and user documentation. It does not require a
parent/child task split because neither new switch is independently releasable:
the runtime, renderer, defaults, and documentation must stay synchronized.

The existing one-way data flow remains unchanged:

```text
public JS defaults + ignored local TOML
  -> constrained TOML parsing and validation
  -> exact constant replacement
  -> atomic ignored .local.js
  -> Clash Verge Rev Global Extend Script
```

No Node-only API, dependency, new proxy group, or new chain hop is introduced
into `clash-verge-ai-residential.js`.

## Public Switch Contract

Add the following rows in the bounded user-configuration section and the
renderer-owned `SWITCH_CONFIG_FIELDS` mapping:

| TOML key | JavaScript constant | Public/example default | Local selected value |
| --- | --- | --- | --- |
| `routing.openai_auth` | `ROUTE_OPENAI_AUTH` | `false` | `true` |
| `routing.openai_web_assets` | `ROUTE_OPENAI_WEB_ASSETS` | `false` | `false` |

`routing.antigravity_google_auth` and `routing.ai_process_fallback` remain
`false` locally. Omitted keys in legacy TOML are appended with the example
defaults by the existing text-preserving completion path. Existing comments,
values, credentials, line endings, and trailing-newline behavior remain
unchanged.

The public script and root `package.json` advance from 5.10.1 to 5.11.0 because
the new opt-in controls are backward-compatible features. The residential
monitor has an independent version contract and is not part of this bump.

## Domain Catalogs And Activation

Add three frozen catalogs:

```js
const OPENAI_AUTH_SUFFIX_DOMAINS = ["auth.openai.com"];
const OPENAI_AUTH_EXACT_DOMAINS = ["auth0.openai.com"];
const OPENAI_WEB_ASSET_SUFFIX_DOMAINS = ["oaistatic.com"];
```

Activation rules:

- `activeSuffixDomains()` includes `OPENAI_AUTH_SUFFIX_DOMAINS` only when
  `ROUTE_OPENAI_AUTH` is true.
- `activeExactDomains()` includes `OPENAI_AUTH_EXACT_DOMAINS` only when
  `ROUTE_OPENAI_AUTH` is true.
- `activeSuffixDomains()` includes `OPENAI_WEB_ASSET_SUFFIX_DOMAINS` only when
  `ROUTE_OPENAI_WEB_ASSETS` is true.
- Neither new switch activates `OPENAI_SHARED_*`, `OPENAI_CORE_*`, or the other
  new switch.

Rule shapes are deliberately bounded:

- `DOMAIN-SUFFIX,auth.openai.com,AI-家宽` covers both the apex and
  `setup.auth.openai.com` without matching `www.openai.com`.
- `DOMAIN,auth0.openai.com,AI-家宽` is exact.
- `DOMAIN-SUFFIX,oaistatic.com,AI-家宽` is independent web-asset routing.
- `DOMAIN-SUFFIX,openai.com` is forbidden.

## Managed Cleanup And DNS Symmetry

All three catalogs are always included in `allPossibleSuffixDomains()` or
`allPossibleExactDomains()`, regardless of current switch values. This lets
`buildManagedRuleSet()` remove output created by an earlier enabled render when
the user later disables a switch.

The active catalogs feed both `buildInjectedRules()` and
`buildNameserverPolicy()`. Expected DNS keys are:

- `+.auth.openai.com` when authentication is enabled;
- `auth0.openai.com` when authentication is enabled;
- `+.oaistatic.com` when web assets are enabled.

Disabling a switch removes only those current-script managed rules and policy
keys. Unknown user-authored rules targeting `AI-家宽` stay preserved under the
existing ownership contract. Repeated `main()` execution remains idempotent.

## Local Rendering

The renderer mapping is the sole owner of TOML table/key, JavaScript constant,
and type. Adding the two mapping rows automatically participates in:

- accepted-key validation;
- boolean validation;
- missing-key completion from the tracked example;
- exact-one constant injection;
- mapping/default/document consistency tests.

After tracked changes are complete, run the renderer against the existing
ignored local TOML. Completion first adds both keys as `false`; update only
`routing.openai_auth` to `true`, render again, and verify selected booleans
without displaying any endpoint, username, or password. Neither ignored local
artifact is staged or committed.

## Windows-To-Ubuntu Deployment Contract

The rendered `.local.js` is a self-contained CommonJS-compatible Clash Verge
global extension. It contains no local filesystem path or shell dependency, so
the same bytes may be copied from Windows to Ubuntu. The TOML renderer does not
need to run on Ubuntu when the already rendered JavaScript is copied.

The rendered JavaScript embeds the residential endpoint and authentication
credentials from the ignored TOML, so it is itself a sensitive artifact. Move
it only through a trusted channel, restrict read access, and never commit it,
publish it, or expose it in logs. Avoiding a second copy of the TOML does not
make the rendered script safe to share.

Portability does not make the surrounding Profile identical. Ubuntu still must
provide the selected `dialer-proxy` name (or an equivalent unambiguous group),
reachable airport nodes, UDP compatibility, and the same Clash Verge/Mihomo
host contract. Cross-platform Node CI verifies syntax and repository behavior;
loading the copied script in Ubuntu Clash Verge and observing sanitized
Connections remains an explicit manual `UNVERIFIED` check.

## Test Matrix

### Public Defaults

- Both new constants are false.
- Auth and asset rules/DNS keys are absent.
- Existing OpenAI core behavior and all current negative cases are unchanged.

### Independent Opt-In

- Auth only: `auth.openai.com`, `setup.auth.openai.com`, a representative
  bounded child, and exact `auth0.openai.com` route through `AI-家宽`.
  `oaistatic.com`, `www.openai.com`, `oaistatsig.com`, and shared dependencies
  remain outside.
- Web assets only: `oaistatic.com` and a representative child route through
  `AI-家宽`; auth hosts remain outside.
- Both: rules and DNS keys are present once each, with no duplicates.

### Ownership And Rendering

- Switch true -> false removes previously generated managed rules and DNS keys
  while preserving an unknown user-authored `AI-家宽` rule.
- Two executions of `main()` produce stable rule and DNS output.
- A legacy/partial local TOML gains both keys at `false` without rewriting
  existing content or line endings.
- An isolated generated-script probe proves `openai_auth = true` changes actual
  route/DNS behavior while the public template remains false and unchanged.
- Mapping-driven tests prove both keys exist in the example and switch docs and
  their documented defaults equal the public constants.

## Documentation

Update README/version, `docs/configuration.md`,
`docs/local-configuration.md`, `docs/routing-scope.md`, and CHANGELOG. The docs
must distinguish:

- first-party auth (`openai_auth`);
- first-party web assets (`openai_web_assets`);
- third-party/shared dependencies (`openai_shared_dependencies`).

The local-configuration guide also states that users may copy the rendered
JavaScript from Windows to Ubuntu without copying the TOML, provided the Ubuntu
Profile exposes the same upstream name and capabilities. It must explicitly
warn that the rendered JavaScript embeds the same endpoint and credentials and
therefore requires secure transfer and secret-file handling.

Documentation must state that enabling `openai_auth` reduces the first-party
auth/core exit split but does not prove that every redirect, SSO, challenge, or
support dependency shares the residential exit. Real login behavior remains
`UNVERIFIED` until a sanitized Connections capture exists.

## Risks And Rollback

- A provider may change its login host inventory. The catalogs are bounded to
  the current official allowlist evidence and require new evidence before
  expansion.
- Users enabling auth routing may consume the residential link for account
  flows and still see a third-party exit split. This is documented, not hidden.
- Public defaults remain false, so upgrade behavior is unchanged unless a user
  opts in or uses the task-authorized ignored local override.
- Roll back the tracked feature as one unit: runtime catalogs/switches,
  renderer mapping/example, docs, tests, and version. Locally, setting
  `openai_auth = false` and rendering is the immediate reversible rollback.
- Preserve the unrelated existing edit to
  `residential-monitor/src-tauri/Cargo.toml`; the task does not touch monitor
  version files or UI/runtime code.
