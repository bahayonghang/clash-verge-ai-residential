# Routing Scope

The residential link is reserved for core AI product traffic. A domain is not included merely because an AI web page loads it.

## Included categories

| Product | Included traffic |
|---|---|
| Claude / Anthropic | Claude product domains, Messages API, MCP/session content, and official inbound IP fallbacks |
| ChatGPT / OpenAI | ChatGPT product domain, OpenAI model API, and uploaded/generated user content |
| Gemini | Gemini Web, Google AI Studio product RPC/streaming hosts, Gemini Developer API, and Vertex AI regional/global model endpoints |
| Google Antigravity / Gemini Code Assist | Product domain and product-specific Code Assist/agent APIs |
| Cursor | Optional Chat/API, Tab, Agent, repository indexing, Cloud Agent/Bugbot API, and product-specific authentication; `routing.cursor_core` is `false` by default |

Cursor support remains available, but v5.5 does not inject Cursor rules or Cursor DNS policy unless `routing.cursor_core = true` is rendered through the local TOML.

## Explicit exclusions

The following classes stay on the original Profile route:

- Cursor Marketplace, extension installation, application downloads, CDN, updates, Remote-SSH/WSL server assets, website, documentation, and forum.
- YouTube, Maps, Google Search, Google Fonts, Gstatic, advertising, analytics, and other generic Google services.
- OpenAI/Claude customer support, telemetry, feature flags, fraud prevention, payment, and other shared third-party infrastructure.
- Public DoH/DoT, generic STUN/TURN, and broad UDP port captures.
- Process-wide routing for Cursor, Claude, ChatGPT, and Antigravity.

## Acceptance rule for new domains

A new domain should be added only when all conditions hold:

1. It is used by model inference, response streaming, AI chat/session control, agent/tool execution, code completion, or repository indexing.
2. The evidence is an official document or a sanitized connection record tied to a reproducible feature.
3. The narrowest practical match can be expressed with `DOMAIN`, a constrained `DOMAIN-SUFFIX`, or a bounded `DOMAIN-REGEX`.
4. Negative tests prove that marketplace, update, download, media, advertising, analytics, and unrelated shared traffic remain outside `AI-家宽`.

## Authentication exit split

Shared login hosts remain on the original Profile by default. In particular, `auth.openai.com` and `accounts.google.com` are not added to the residential route, while core chat/model traffic uses the residential exit. Strict risk-control systems can therefore observe different login and model-traffic IPs and may request additional verification. This is an intentional narrow-scope trade-off, not a reason to add either shared authentication domain without evidence.

## Managed-rule ownership after v5.5

The script replaces rules that the current version can generate, including output from a switch that was later disabled. It no longer contains automatic migration lists for pre-v5.4 broad rules or retired v5.4 Cursor entries. Unknown rules targeting `AI-家宽` are treated as user-owned and preserved. If generated output was manually persisted in a subscription or Global Extend Config (Merge), remove stale entries there using the exact search list in [Troubleshooting](troubleshooting.md), then refresh the Profile.
