# DNS and leak model

## Data paths

```text
AI application request
  -> Mihomo rule match
  -> AI-家宽
  -> 家宽-SOCKS5
  -> current Profile upstream selected by dialer-proxy
  -> residential exit
  -> AI service
```

The transport connection to the residential SOCKS5 server is dialed through the selected airport group. The external service sees the residential exit. When chaining is required, the local machine does not connect to the residential endpoint directly.

## DNS paths

```text
AI domain query       -> residential DoH via AI-家宽
Other overseas query  -> non-AI DoH via current Profile upstream
Chinese domain query  -> domestic DoH via DIRECT
Private/LAN query     -> system resolver
Proxy-server lookup   -> bootstrap/direct resolver to avoid recursion
```

This is intentionally different from a global DNS-leak-test configuration. A generic DNS leak test may show the airport or domestic resolver because non-AI queries are not sent through the residential route. The invariant is narrower: AI domain resolution and AI application connections must use the intended residential path.

Timing: under `enhanced-mode: fake-ip`, most A/AAAA queries are answered from the fake-ip pool and never hit an upstream resolver. `nameserver-policy` is therefore mainly a fallback for fake-ip-filter hits, non-A/AAAA query types, and real-IP lookups needed by L3 outbound. SOCKS5 outbound forwards the hostname (RFC 1928 domain addressing), so AI connections usually let the residential SOCKS5 server do the recursive resolve. That is expected. When Mihomo must resolve an AI domain itself, the policy entries still keep that lookup on the residential side.

## Fields Clash Verge Rev restores

Current Clash Verge Rev saves control-plane fields (`tun`, `ipv6`, `mode`, ports, and similar) before the global script runs, then restores them afterward. Script-side `hardenTun` DNS-hijack completion and `config.ipv6 = false` therefore do not take effect on these hosts. The script keeps that logic for older hosts and logs an `info` line. Turn IPv6 off in the settings page and configure DNS hijack in TUN settings. DNS servers, `nameserver-policy`, and fake-ip rebuilt by the script are kept; if Clash Verge Rev DNS override is enabled, `dns.ipv6` is also restored from app settings.

## geosite dependency

`geosite:cn` and `geosite:private` in `nameserver-policy` need `geosite.dat`. Mihomo downloads that file on first use. A brand-new offline install fails configuration parsing if the download fails. Clash Verge Rev shows a validation error; on first launch the app falls back to a minimal default config. Most subscriptions already include geo rules that trigger the same download, so the risk is mainly a fresh offline install. Recovery is in [Troubleshooting](troubleshooting.md).

## Strict-DNS performance trade-off

The script rebuilds DNS policy instead of inheriting arbitrary subscription paths. Real non-AI overseas lookups go through DoH bound to the current Profile upstream. The first real lookup for a new domain on a GEOIP fallback can add roughly one airport round trip; cache hits do not pay that setup cost. The trade-off stays for resolver consistency and pollution resistance.

## Login and model exit split

Shared authentication hosts are outside the default AI-only scope. `auth.openai.com` and `accounts.google.com` therefore use the original Profile, while core chat/model requests use the residential exit. A strict risk-control system can observe different login and model-traffic IPs and request additional verification. The script does not add either shared authentication host merely to hide this split. Opt-in shared-dependency switches only with evidence and an understood scope.

## What the script mitigates

- DNS divergence between AI application traffic and AI domain resolution.
- Accidental residential routing of unrelated media, marketplace, download, and shared-service traffic.
- Recursive chaining through `include-all` groups.
- IPv6 use inside Mihomo configuration.
- Missing TUN DNS interception entries when TUN is already enabled.

## What the script cannot guarantee alone

- Operating-system traffic that bypasses Clash Verge Rev.
- Browser or application private DoH to an unknown endpoint.
- IPv6 leaks outside Mihomo/TUN routing. On current Clash Verge Rev hosts, also check the app IPv6 switch, because the host restores the script's `ipv6: false`.
- UDP support of the currently selected provider node. Airport subscription nodes often omit `udp`, and Mihomo treats that as `false`; unless the provider sets `udp: true`, the chain silently drops UDP.
- Runtime selector choices such as `DIRECT`.
- WebRTC behavior of arbitrary applications when shared STUN/TURN capture is disabled.

Use the operating system firewall, TUN mode, browser DNS settings, and sanitized connection inspection as complementary controls.
