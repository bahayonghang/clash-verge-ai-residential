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
Enabled exact/suffix AI query -> residential DoH via AI-家宽
Other overseas query  -> non-AI DoH via current Profile upstream
Chinese domain query  -> domestic DoH via DIRECT
Private/LAN query     -> system resolver
Proxy-server lookup   -> bootstrap/direct resolver to avoid recursion
```

This is intentionally different from a global DNS-leak-test configuration. A generic DNS leak test may show the airport or domestic resolver because non-AI queries are not sent through the residential route. The configuration assigns residential resolvers to enabled exact/suffix AI domains. Regex domains have the exception below, and actual paths still depend on host settings.

Timing: under `enhanced-mode: fake-ip`, most A/AAAA queries are answered from the fake-ip pool and never hit an upstream resolver. `nameserver-policy` is therefore mainly a fallback for fake-ip-filter hits, non-A/AAAA query types, and real-IP lookups needed by L3 outbound. SOCKS5 outbound forwards the hostname (RFC 1928 domain addressing), so AI connections usually let the residential SOCKS5 server do the recursive resolve. That is expected. When Mihomo must resolve a domain itself, its exact/suffix policy specifies residential resolution; this coverage does not extend to regex-only hosts.

## Regex domains and configuration errors

Regional Vertex `DOMAIN-REGEX` routes and enabled Cursor `repo[0-9]+.cursor.sh` routes have no equivalent `nameserver-policy`. Local real lookups for those hosts may use the default non-AI DoH. Fake-IP responses or SOCKS domain forwarding do not prove every query uses the same exit. `respect-rules` governs DNS connections; it does not translate application-domain regexes into resolver policies. The script keeps narrow matching without adding broad `+.googleapis.com` or `+.cursor.sh` policies. Actual DNS/UDP paths remain UNVERIFIED.

A successfully generated `AI-家宽` group has only the residential SOCKS5 member. Extra member sources or filtering fields on an existing same-name group cause an error. However, Clash Verge Rev may discard script output after an exception and use the original Profile. Rejection does not prove traffic is blocked. Correct the error, confirm the configuration is active, then check the application chain. A static public IP provided by the SOCKS service also needs separate verification.

## Fields Clash Verge Rev restores

Current Clash Verge Rev saves the fields that its settings page manages before the global script runs, then restores them afterward. These fields are:

- Top-level control-plane fields: `external-controller*`, `secret`, the port fields, `mode`, `allow-lan`, `log-level`, `ipv6`, and `unified-delay`.
- `tun.enable`, and the saved TUN settings fields: `stack`, `device`, `auto-route`, `route-exclude-address`, `auto-redirect` (Linux), `auto-detect-interface`, `dns-hijack`, `strict-route`, and `mtu`.
- When DNS override is on: all non-empty `dns.*` fields in `dns_config.yaml`, and a non-empty `hosts`.

From Clash Verge Rev v2.5.5, when an extension changes one of these fields, the app shows the notice "Extensions wrote …, which Settings manages, so those values were discarded" and discards the written value.

The script therefore does not write any `tun` key or the top-level `ipv6`. It only reads these fields and logs a `warn` when a value is unsafe:

- TUN is on and `dns-hijack` does not contain `any:53` or `tcp://any:53`. The script skips this check when `runtime.harden_existing_tun_dns_hijack = false`.
- `runtime.harden_existing_tun_dns_hijack` and `runtime.enable_tun_strict_route` are both `true`, TUN is on, and `strict-route` is not on.
- The top-level `ipv6` is `true`.

Turn IPv6 off in the settings page and configure DNS hijack in TUN settings. When DNS override is off, the DNS servers, `nameserver-policy`, and fake-ip that the script rebuilds are kept. When DNS override is on, the settings page owns all non-empty `dns.*` fields in `dns_config.yaml`. The host discards script changes to those fields and can show a `dns.*` notice.

## geosite dependency

`geosite:cn` and `geosite:private` in `nameserver-policy` need `geosite.dat`. Mihomo downloads that file on first use. A brand-new offline install fails configuration parsing if the download fails. Clash Verge Rev shows a validation error; on first launch the app falls back to a minimal default config. Most subscriptions already include geo rules that trigger the same download, so the risk is mainly a fresh offline install. Recovery is in [Troubleshooting](troubleshooting.md).

## Strict-DNS performance trade-off

The script rebuilds DNS policy instead of inheriting arbitrary subscription paths. Real non-AI overseas lookups go through DoH bound to the current Profile upstream. The first real lookup for a new domain on a GEOIP fallback can add roughly one airport round trip; cache hits do not pay that setup cost. The trade-off stays for resolver consistency and pollution resistance.

## Login and model exit split

Shared authentication hosts are outside the default AI-only scope. `auth.openai.com` and `accounts.google.com` therefore use the original Profile, while core chat/model requests use the residential exit. A strict risk-control system can observe different login and model-traffic IPs and request additional verification. The script does not add either shared authentication host merely to hide this split. Opt-in shared-dependency switches only with evidence and an understood scope.

## What the script mitigates

- Configured DNS divergence for enabled exact/suffix AI domains.
- Accidental residential routing of unrelated media, marketplace, download, and shared-service traffic.
- Recursive chaining through `include-all` groups.
- IPv6 queries at the DNS layer (the rebuilt `dns.ipv6: false`, effective when DNS override is off).
- Top-level `ipv6` on, or missing TUN DNS interception entries when TUN is on: the script logs a `warn`, and the user changes the value in the settings page.

## What the script cannot guarantee alone

- Operating-system traffic that bypasses Clash Verge Rev.
- Browser or application private DoH to an unknown endpoint.
- IPv6 leaks outside Mihomo/TUN routing. The Clash Verge Rev settings-page IPv6 switch sets the top-level `ipv6`. The script does not write that field.
- UDP support of the currently selected provider node. Airport subscription nodes often omit `udp`, and Mihomo treats that as `false`; unless the provider sets `udp: true`, the chain silently drops UDP.
- Runtime selector choices such as `DIRECT`.
- WebRTC behavior of arbitrary applications when shared STUN/TURN capture is disabled.

Use the operating system firewall, TUN mode, browser DNS settings, and sanitized connection inspection as complementary controls.
