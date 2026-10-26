# Design

## Boundary

Add four exact sign-in hosts to the existing `grok_core` list in `clash-verge-ai-residential.js`. No new routing switch. No suffix for `x.ai`, `spacex.ai`, `grokusercontent.com`, or `grokipedia.com`.

Hosts:

- `accounts.x.ai`
- `accounts.spacex.ai`
- `auth.grokusercontent.com`
- `auth.grokipedia.com`

They go in `GROK_EXACT_DOMAINS` next to `auth.x.ai`. `grokActiveExactDomains()` already returns that array whenever `ROUTE_GROK_CORE` is true, including when `ROUTE_GROK_WEB_ASSETS` is false. `allPossibleExactDomains()` already spreads `GROK_EXACT_DOMAINS`, so turning the switch off removes the managed rules and the bare DNS keys.

## Data flow

`buildDomainRules` emits `DOMAIN,<host>,AI-家宽`. `residentialDnsExactSet` emits a bare `nameserver-policy` key per exact host. It must not emit `+.x.ai`, `+.spacex.ai`, `+.grokusercontent.com`, or `+.grokipedia.com`.

`skills/residential-rule-tuning/scripts/build-inputs.js` already maps `GROK_EXACT_DOMAINS` to `grok_core` and keeps `GROK_STRICT_EXACT_DOMAINS` on `grok_web_assets`. The supported-switch count stays 13 of 26.

## Why exact

The 2026-10-26 TLS check (system resolver, FlClash fake-IP path) got an authorized certificate for each host. Every certificate is a parent wildcard. `accounts.x.ai` and `slack.hooks.x.ai` share `*.x.ai`. A suffix rule would send the install host `x.ai` and the Slack hook host through residential. The official page prefers wildcards for a firewall allowlist. This repository's residential list is narrower: exact host only.

Live Grok Bot / `grok.exe` sockets from the same day are already covered (`api2.cursor.sh`, `api3.cursor.sh`, `grok.com`, `cli-chat-proxy.grok.com`). This change does not add them again.

## Compatibility

- Default rule count moves from 55 to 59 (`tests/regression.test.js`).
- Existing local TOML does not need a new key. `routing.grok_core = true` already turns the list on.
- Do not render or edit `*.local.js`. The running FlClash copy does not change until the user reloads the extension.

## Rollback

Remove the four strings from `GROK_EXACT_DOMAINS` and the matching tests/docs. No migration of stored profiles: the script cleans managed rules it no longer emits.

## Tradeoff

These four hosts were not in today's connection table. The user accepted official-doc admission instead of waiting for a new capture. Sibling hosts that share the wildcard certificate stay excluded.
