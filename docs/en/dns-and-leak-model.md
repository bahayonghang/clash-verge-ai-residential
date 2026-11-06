# DNS and leak model

## Data paths

```text
AI application request
  -> Mihomo rule match
  -> AI-家宽
  -> connect to the residential endpoint via the dialer-proxy Profile upstream
  -> 家宽-SOCKS5 / residential exit
  -> AI service
```

The transport connection to the residential SOCKS5 server is dialed through the selected airport group. The external service sees the residential exit. When chaining is required, the local machine does not connect to the residential endpoint directly.

## DNS paths

```text
Enabled exact/suffix/regex AI query -> residential DoH via AI-家宽
Other overseas query  -> non-AI DoH via current Profile upstream
Chinese domain query  -> domestic DoH via DIRECT
Private/LAN query     -> system resolver
Proxy-server lookup   -> bootstrap/direct resolver to avoid recursion
```

This is intentionally different from a global DNS-leak-test configuration. A generic DNS leak test may show the airport or domestic resolver because non-AI queries are not sent through the residential route. Enabled exact/suffix AI domains receive residential policies; regional Vertex and optional Cursor indexing regexes use a dedicated local ruleset. Actual paths still depend on host settings.

Extra sites can intentionally separate application routing from DNS: `residentialDns: false`, as used for AnyRouter, injects residential application routes without residential DNS policies. The residential DNS path above applies only to domains with a generated policy; an application's rule target does not establish its DNS outbound.

Timing: under `enhanced-mode: fake-ip`, most A/AAAA queries are answered from the fake-ip pool and never hit an upstream resolver. `nameserver-policy` is therefore mainly a fallback for fake-ip-filter hits, non-A/AAAA query types, and real-IP lookups needed by L3 outbound. SOCKS5 outbound forwards the hostname (RFC 1928 domain addressing), so AI connections usually let the residential SOCKS5 server do the recursive resolve. That is expected. When Mihomo must resolve a domain itself, exact/suffix entries and the regex ruleset policy specify residential resolution.

## Regex DNS ruleset and configuration errors

Regional Vertex `DOMAIN-REGEX` routes and enabled Cursor `repo[0-9]+.cursor.sh` routes also generate `AI-家宽-DNS-REGEX`, an inline/classical ruleset whose payload comes directly from existing application regexes. It downloads no remote rules. `nameserver-policy` uses `rule-set:AI-家宽-DNS-REGEX` to select residential DoH. `vertex_ai_endpoints` and `cursor_repository_indexing` control their respective patterns; disabling both removes the managed ruleset and policy. The script adds no broad `+.googleapis.com`, `+.cursor.sh`, in-label wildcard, or raw regex DNS key.

`AI-家宽-DNS-REGEX` is reserved. An existing same-name provider must contain the three generated fields and known regex payload; extra fields, remote sources, or unknown patterns fail validation. Custom `RULE-SET` references (including `sub-rules`), other compound DNS policies, `dns.fake-ip-filter`, `sniffer.skip-domain`, and `sniffer.force-domain` must not reference it, preventing dangling references when switches are disabled. Unknown ordinary providers stay unchanged, and rejected inputs are not mutated.

This mechanism passed real DNS queries and two-hop local SOCKS5 routing in an isolated Mihomo v1.19.32 environment. Validate support on other core versions; fix unsupported versions/configuration instead of broadening suffixes. Isolation does not prove public DoH TLS, the residential provider's UDP support, or a fixed public IP. Fake-IP answers and SOCKS hostname forwarding do not replace query evidence.

When reverting to an older script, start from the original Profile. Older scripts cannot clean up the new provider and policy; do not apply them over newer output.

A successfully generated `AI-家宽` group has only the residential SOCKS5 member. Extra member sources or filtering fields on an existing same-name group cause an error. However, Clash Verge Rev may discard script output after an exception and use the original Profile. Rejection does not prove traffic is blocked. Correct the error, confirm the configuration is active, then check the application chain. A static public IP provided by the SOCKS service also needs separate verification.

## Later client overrides and version boundaries

The inspected Clash Verge Rev v2.5.8 / v2.5.10 saves fields managed by its settings page before the global script runs, then restores them afterward. These fields are:

- Top-level control-plane fields: `external-controller*`, `secret`, the port fields, `mode`, `allow-lan`, `log-level`, `ipv6`, and `unified-delay`.
- `tun.enable`, and the saved TUN settings fields: `stack`, `device`, `auto-route`, `route-exclude-address`, `auto-redirect` (Linux), `auto-detect-interface`, `dns-hijack`, `strict-route`, and `mtu`.
- When DNS override is on: all non-empty `dns.*` fields in `dns_config.yaml`, and a non-empty `hosts`.

In these versions, extension changes to settings-owned fields are discarded and may produce an ownership notice. An owned `nameserver-policy` replaces the entire map, rather than merging individual entries. Check source and final configuration for other versions; floating documentation cannot replace version-specific evidence. See [dual-client setup](configuration.md#dual-client-setup) for pinned sources.

FlClash v0.9.2 continues generating configuration after an ordinary Profile's script runs. It rewrites process lookup and TUN settings, merges selected DNS override fields, and may append `system://` or prepend/replace rules. A selected `nameserver-policy` likewise replaces the whole map. Profile preview exposes this generation result but does not establish what the running core loaded. Its macOS system DNS coordinator is not evidence of Windows/Ubuntu behavior.

The script therefore does not write any `tun` key or the top-level `ipv6`. It only reads these fields and logs a `warn` when a value is unsafe:

- TUN is on and `dns-hijack` does not contain `any:53` or `tcp://any:53`. The script skips this check when `runtime.harden_existing_tun_dns_hijack = false`.
- `runtime.harden_existing_tun_dns_hijack` and `runtime.enable_tun_strict_route` are both `true`, TUN is on, and `strict-route` is not on.
- The top-level `ipv6` is `true`.

Warnings reflect script input, not later host changes. Check final IPv6, DNS interception, and policies before making authorized settings changes; no warning does not mean a pass. In these Verge versions, disabling DNS override prevents that layer from restoring DNS fields; enabling it makes non-empty `dns.*` fields settings-owned. Other extensions can still change output. See [dual-client runtime validation](client-validation.md) for Windows/Ubuntu resolvers, real queries, and recovery.

## geosite dependency

`geosite:cn` and `geosite:private` in `nameserver-policy` need `geosite.dat`. Mihomo downloads that file on first use. A brand-new offline install fails configuration parsing if the download fails. Clash Verge Rev shows a validation error; on first launch the app falls back to a minimal default config. Most subscriptions already include geo rules that trigger the same download, so the risk is mainly a fresh offline install. Recovery is in [Troubleshooting](troubleshooting.md).

## Strict-DNS performance trade-off

The script rebuilds DNS policy instead of inheriting arbitrary subscription paths. Real non-AI overseas lookups go through DoH bound to the current Profile upstream. The first real lookup for a new domain on a GEOIP fallback can add roughly one airport round trip; cache hits do not pay that setup cost. The trade-off stays for resolver consistency and pollution resistance.

## Login and model exit split

`routing.openai_auth` and `routing.antigravity_google_auth` are on by default, so `auth.openai.com` and `accounts.google.com` use the same residential exit as core chat and model requests. `oaistatic.com` and shared dependencies such as WorkOS, Intercom, Stripe, Cloudflare Challenge, Sentry, and Datadog stay on the original Profile. Those third-party redirects can still show a risk-control system a different exit. Enable a shared-dependency switch only with evidence and an understood scope.

## What the script mitigates

- Configured DNS divergence for enabled exact/suffix AI domains, regional Vertex, and optional Cursor indexing regexes.
- Accidental residential routing of unrelated media, marketplace, download, and shared-service traffic.
- Recursive chaining through `include-all` groups.
- IPv6 queries at the DNS layer (the rebuilt `dns.ipv6: false`, subject to final client configuration).
- Top-level `ipv6` on, or missing TUN DNS interception entries when TUN is on: the script logs a `warn`, and the user changes the value in the settings page.

## What the script cannot guarantee alone

- Operating-system traffic that bypasses the active proxy client.
- Browser or application private DoH to an unknown endpoint.
- IPv6 leaks outside Mihomo/TUN routing. The host manages top-level `ipv6`; the script does not write it. Check it separately from `dns.ipv6`.
- UDP support of the currently selected provider node. Airport subscription nodes often omit `udp`, and Mihomo treats that as `false`; unless the provider sets `udp: true`, the chain silently drops UDP.
- Runtime selector choices such as `DIRECT`.
- WebRTC behavior of arbitrary applications when shared STUN/TURN capture is disabled.

Operating system firewalls, TUN, browser DNS settings, and sanitized connection inspection are complementary controls. Preserve recovery steps before changes and verify each item using the [runtime checklist](client-validation.md).
