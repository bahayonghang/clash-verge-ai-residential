# OpenAI Authentication Routing Evidence

## Decision

- Add two independent controls:
  - `routing.openai_auth` / `ROUTE_OPENAI_AUTH`
  - `routing.openai_web_assets` / `ROUTE_OPENAI_WEB_ASSETS`
- Keep both public and example defaults `false`.
- In the ignored current local TOML, enable only `openai_auth`; keep
  `openai_web_assets`, `antigravity_google_auth`, and process fallback disabled.
- Do not change the airport-to-residential `dialer-proxy` chain.

## Repository Evidence

- `ROUTE_OPENAI_SHARED_DEPENDENCIES` and `ROUTE_OPENAI_CORE` are separate
  public switches (`clash-verge-ai-residential.js:126-129`).
- OpenAI core currently owns three suffixes and five exact `chat.openai.com`
  family hosts (`clash-verge-ai-residential.js:217-263`). It does not own the
  first-party authentication or static-asset hosts.
- OpenAI shared dependencies own third-party WorkOS, Intercom, SendGrid,
  Stripe, Cloudflare Challenge, Sentry, and Datadog domains
  (`clash-verge-ai-residential.js:352-373`). Enabling that switch is not an
  equivalent replacement for routing first-party OpenAI auth hosts.
- Active domain builders and all-possible domain builders are separate
  (`clash-verge-ai-residential.js:1192-1273`). A disabled switch therefore
  still needs its catalogs in `allPossible*` so prior managed output can be
  removed safely.
- Managed rules and DNS policy are derived from those builders
  (`clash-verge-ai-residential.js:1414-1483`), so rule and DNS ownership must be
  updated together.
- The current routing-scope document deliberately records an authentication
  exit split (`docs/routing-scope.md:87-89`). The new controls change that from
  a fixed behavior to an explicit opt-in for first-party OpenAI hosts only.

## Domain Classification

The prior official OpenAI network allowlist snapshot in
`.trellis/tasks/archive/2026-08/08-17-ai-domain-routing-audit/research/openai-9247338-allowlist-excerpt.md`
lists `*.auth.openai.com`, `setup.auth.openai.com`, `auth0.openai.com`, and
`*.oaistatic.com`. This allowlist is firewall evidence, not proof that every
listed host should use the residential route.

| Catalog | Rule shape | Reason |
| --- | --- | --- |
| `auth.openai.com` | `DOMAIN-SUFFIX` | Covers the apex and bounded children such as `setup.auth.openai.com` without broad `openai.com`. |
| `auth0.openai.com` | `DOMAIN` | Exact official authentication host; no evidence supports sibling expansion. |
| `oaistatic.com` | `DOMAIN-SUFFIX` | First-party web assets, independently optional from authentication. |

Explicit non-members include `www.openai.com`, unrelated `openai.com`
subdomains, `oaistatsig.com`, `cdn.openaimerge.com`, and all third-party shared
dependencies unless their existing switch is separately enabled.

## Evidence Boundary

- Unit tests can verify rule generation, DNS policy, switch independence,
  cleanup, and TOML rendering.
- They cannot prove a complete browser login redirect chain, account-risk
  behavior, or actual Clash Verge Rev host integration.
- Even with `openai_auth = true`, WorkOS, Cloudflare Challenge, and other shared
  dependencies may remain on the airport exit while their existing shared
  switch is disabled. Therefore the task must not claim complete single-exit
  login behavior.
- A sanitized Connections capture from an actual login remains `UNVERIFIED`.
