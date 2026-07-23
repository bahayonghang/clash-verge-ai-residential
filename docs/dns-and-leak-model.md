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
