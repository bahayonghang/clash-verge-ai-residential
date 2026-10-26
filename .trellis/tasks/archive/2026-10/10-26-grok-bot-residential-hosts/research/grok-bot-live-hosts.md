# Grok Bot live hosts (2026-10-26)

Sanitized local evidence. No proxy endpoints, credentials, or account identifiers.

## Method

FlClash is the active TUN client. Its running config has `external-controller` empty, so `/connections` could not be queried. Grok Bot and `grok.exe` established sockets land on `198.18.0.0/15`, which is that TUN fake-IP pool. That shows the entry gate is open: the traffic is inside the proxy, not a direct physical NIC path.

Each candidate name was resolved through the system resolver. The answer was a fake-IP. Matching that address to a live socket maps the connection to a hostname without reading TLS payloads. Two sockets from the first snapshot (`198.18.1.178`, `198.18.1.235`) closed before they could be mapped. They are not admission evidence.

ResiWatch `monitor.sqlite3` process rows for `Grok Bot.exe`, `grok.exe`, and `grok-app.exe` stop at unix `1789864784` (about 2026-09-19). That history is not current traffic.

`docs.x.ai` itself resolves to fake-IP `198.18.4.238` from this machine, so the official pages below were read from the public search index, not a direct HTTP fetch.

## Live sockets

Second snapshot, same session as the DNS map:

| Process | Fake-IP | Resolved name | Already residential? |
| --- | --- | --- | --- |
| `grok.exe` | `198.18.1.177:443` | `grok.com` | Yes. `DOMAIN-SUFFIX,grok.com,AI-家宽` while `routing.grok_web_assets` is on |
| `grok.exe` | `198.18.0.159:443` | `cli-chat-proxy.grok.com` | Yes. Same `grok.com` suffix. Also in `GROK_STRICT_EXACT_DOMAINS` |
| `Grok Bot.exe` | `198.18.0.117:443` | `api2.cursor.sh` | Yes. `DOMAIN,api2.cursor.sh,AI-家宽` |

An earlier snapshot also had `Grok Bot.exe` to `198.18.0.118`, which resolves to `api3.cursor.sh` (`CURSOR_EXACT_DOMAINS`).

Running `config.yaml` sends those rules to proxy group `AI-家宽`. That group is `select` and its only member is the residential SOCKS5 node. It is not `DIRECT`.

Public template switches involved: `ROUTE_GROK_CORE`, `ROUTE_GROK_WEB_ASSETS`, `ROUTE_CURSOR_CORE` (all default true). Local `routing.grok_core`, `routing.grok_web_assets`, and `routing.cursor_core` are true. `routing.cursor_repository_indexing` and `routing.cursor_process_fallback` are false.

## Names seen in the app, not in the live socket table

Grok Bot `0.75.8` (`C:\Program Files\Grok Bot\Grok Bot.exe`). Bundle `app.asar` and `sand-statsig-bootstrap.json` mention:

- `auth.grok.com`, `connectors-gateway.grok.com`: covered by `DOMAIN-SUFFIX,grok.com` when web assets are on. Not in `GROK_STRICT_EXACT_DOMAINS`. Not connected during this capture.
- `api.x.ai`, `auth.x.ai`: already in the template. Not connected during this capture.
- Session-storage computer names under `*.cursorvm.com`: covered by `DOMAIN-SUFFIX,cursorvm.com`. The six names resolved during this pass were not the live sockets.

## Rejected for the residential list

Observed historically in ResiWatch, or present as bundle/CDN/telemetry names. Repo policy rejects broad provider suffixes, marketplace/CDN, and telemetry. The running template comments already exclude `cursor-cdn.com`, `downloads.cursor.com`, `api.mixpanel.com`, apex `x.ai`, and shared object storage.

- `api.mixpanel.com`, `metrics.cursor.sh`, `api.statsigcdn.com`, `statsigapi.net`
- `cursor-cdn.com`, `downloads.cursor.com`
- `x.ai` (install host; suffix `x.ai` is intentionally not used)
- `public.blob.vercel-storage.com` and specific blob hosts
- `grok-bot-templates.s3.us-east-1.amazonaws.com`
- `github.com`, `api.github.com`, `avatars.githubusercontent.com`, `codeload.github.com`, `release-assets.githubusercontent.com`
- `accounts.google.com`, `fonts.gstatic.com`, `dl.google.com`, `pbs.twimg.com`

Official firewall pages also mention `*.cursor-cdn.com`, `*.cursorapi.com`, and `downloads.cursor.com`. Those are allowlist entries for a filtering gateway. They are not new residential AI routes.

## Official pages (search index, 2026-10-26)

- <https://docs.x.ai/grok-bot/proxies>: Grok Bot desktop talks to `*.cursor.sh` (chat, sign-in, approvals) and a nested `*.*.cursorvm.com` computer host. `DOMAIN-SUFFIX,cursorvm.com` matches both label depths. This repo does not take all of `*.cursor.sh`; it keeps the narrow exact/suffix set. Live chat was `api2.cursor.sh`, which is in that set.
- <https://docs.x.ai/x-subscription/sign-in-domains>: On 2026-10-30, Cursor sign-in moves from `authenticator.cursor.sh` to `accounts.spacex.ai` and `accounts.x.ai`. Minimum exact hosts listed there include `accounts.x.ai`, `accounts.spacex.ai`, `auth.x.ai`, `auth.grok.com`, `auth.grokusercontent.com`, `auth.grokipedia.com`, and `grok.com`.
- <https://docs.x.ai/x-subscription/grok-bot-sign-in>: Grok Bot uses that shared sign-in path.

Coverage of those minimum hosts today:

| Host | Covered now? |
| --- | --- |
| `grok.com`, `auth.grok.com` | Yes, via `grok.com` suffix |
| `auth.x.ai` | Yes, exact |
| `accounts.x.ai` | No. `api.x.ai` suffix does not match `accounts.x.ai` |
| `accounts.spacex.ai` | No |
| `auth.grokusercontent.com` | No |
| `auth.grokipedia.com` | No |

`*.x.ai` and `*.spacex.ai` would also pull in hosts this repo has refused (apex `x.ai`, and any future shared SpaceX/xAI property). They are not a candidate rule shape.

## TLS check (2026-10-26, local resolver)

`node:tls` to port 443 with SNI set to the hostname. Resolution went through the system resolver, which returned FlClash fake-IP, so this is the current proxy path rather than a separate residential-DoH vantage. All four handshakes were `authorized=true`. Each certificate is a parent wildcard, which is why the rule must stay `DOMAIN` exact:

| Host | Certificate names |
| --- | --- |
| `accounts.x.ai` | `x.ai`, `*.x.ai`, `slack.hooks.x.ai` |
| `accounts.spacex.ai` | `spacex.ai`, `*.spacex.ai` |
| `auth.grokusercontent.com` | `grokusercontent.com`, `*.grokusercontent.com` |
| `auth.grokipedia.com` | `grokipedia.com`, `*.grokipedia.com` |

`slack.hooks.x.ai` is on the same certificate as `accounts.x.ai` and stays off the residential list. Parent apex names (`x.ai`, `spacex.ai`, `grokusercontent.com`, `grokipedia.com`) stay off too.
