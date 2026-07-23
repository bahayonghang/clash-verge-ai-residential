# DNS and Leak Model

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

The transport connection to the residential SOCKS5 server is dialed through the selected airport group. The external service sees the residential exit, while the local machine does not directly connect to the residential endpoint when chaining is required.

## DNS paths

```text
AI domain query       -> residential DoH via AI-家宽
Other overseas query -> non-AI DoH via current Profile upstream
Chinese domain query -> domestic DoH via DIRECT
Private/LAN query    -> system resolver
Proxy-server lookup  -> bootstrap/direct resolver to avoid recursion
```

This is intentionally different from a global DNS-leak-test configuration. A generic DNS leak test may show the airport or domestic resolver because non-AI queries are not sent through the residential route. The invariant is narrower: AI domain resolution and AI application connections must use the intended residential path.

## Strict-DNS performance trade-off

The script deliberately rebuilds DNS policy instead of inheriting arbitrary subscription paths. Real non-AI overseas lookups are sent through DoH bound to the current Profile upstream. When a GEOIP fallback needs the first real lookup for a new domain, establishing that path can add roughly one airport round trip; subsequent cache hits do not pay the same lookup cost. v5.5 retains this trade-off for resolver consistency and pollution resistance.

## Login and model exit split

Shared authentication hosts are outside the default AI-only scope. `auth.openai.com` and `accounts.google.com` therefore use the original Profile, while core chat/model requests use the residential exit. A strict risk-control system can observe different login and model-traffic IPs and request additional verification. The script does not add either shared authentication host merely to hide this split; opt-in shared-dependency switches should be enabled only with evidence and an understood scope.

## What the script mitigates

- DNS divergence between AI application traffic and AI domain resolution.
- Accidental residential routing of unrelated media, marketplace, download, and shared-service traffic.
- Recursive chaining through `include-all` groups.
- IPv6 use inside Mihomo configuration.
- Missing TUN DNS interception entries when TUN is already enabled.

## What the script cannot guarantee alone

- Operating-system traffic that bypasses Clash Verge Rev.
- Browser or application private DoH to an unknown endpoint.
- IPv6 leakage outside Mihomo/TUN routing.
- UDP support in the currently selected provider node.
- Runtime selector choices such as `DIRECT`.
- WebRTC behavior of arbitrary applications when shared STUN/TURN capture is disabled.

Use the operating system firewall, TUN mode, browser DNS settings, and sanitized connection inspection as complementary controls.
