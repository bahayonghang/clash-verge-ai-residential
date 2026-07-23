# Routing Scope

The residential link is reserved for core AI product traffic. A domain is not included merely because an AI web page loads it.

## Included categories

| Product | Included traffic |
|---|---|
| Claude / Anthropic | Claude product domains, Messages API, MCP/session content, and official inbound IP fallbacks |
| ChatGPT / OpenAI | ChatGPT product domain, OpenAI model API, and uploaded/generated user content |
| Gemini | Gemini Web, Google AI Studio product RPC/streaming hosts, Gemini Developer API, and Vertex AI regional/global model endpoints |
| Google Antigravity / Gemini Code Assist | Product domain and product-specific Code Assist/agent APIs |
| Cursor | Chat/API, Tab, Agent, repository indexing, Cloud Agent/Bugbot API, and product-specific authentication |

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

## Legacy cleanup lists

`LEGACY_V53_*` constants contain previously injected broad domains only so the script can remove them during migration. Their presence in source code does not make them active routing rules. `buildInjectedRules()` and the regression tests define the active scope.
