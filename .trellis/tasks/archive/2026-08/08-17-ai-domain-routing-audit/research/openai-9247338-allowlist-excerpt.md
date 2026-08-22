# OpenAI 官方 allowlist 摘录

- 页面：https://help.openai.com/en/articles/9247338-network-recommendations-for-chatgpt-errors-on-web-and-apps
- 标题：Network recommendations for ChatGPT errors on web and apps
- 页面自称 Updated: 14 days ago（取回时相对日期，不是绝对日）
- 取回：2026-08-17，方法 `exa__web_fetch_exa`
- 本机直接 `web_fetch` 失败：Clash fake-ip 将 `help.openai.com` 解析到 `198.18.x`，触发 SSRF 拦截
- Codex 隔离浏览器：403 / Cloudflare challenge

以下为 allowlist 与 Voice 小节原文摘录，未改写。`*.openai.com` 等宽项仍受仓库准入规则约束，不因出现在官方列表就注入。

## OpenAI/ChatGPT domains to allowlist

```
*.auth.openai.com
*.chatgpt.com
*.ct.sendgrid.net
*.intercom.io
*.intercomcdn.com
*.oaistatic.com
*.oaiusercontent.com
*.openai.com
*.oaistatsig.com
android.chat.openai.com
auth0.openai.com
cdn.openaimerge.com
cdn.workos.com
challenges.cloudflare.com
chat.openai.com
desktop.chat.openai.com
forwarder.workos.com
humb.apple.com
images.workoscdn.com
ios.chat.openai.com
js.intercomcdn.com
js.stripe.com
o207216.ingest.sentry.io
o33249.ingest.sentry.io
rum.browser-intake-datadoghq.com
setup.auth.openai.com
setup.workos.com
tcr9i.chat.openai.com
workos.imgix.net
```

## WebSocket

- ChatGPT: `wss://ws.chatgpt.com`
- Codex: `wss://chatgpt.com/`

## ChatGPT Voice firewall settings

原文：ChatGPT Voice connects to OpenAI servers over UDP port 3478. The current server IP address ranges are listed in chatgpt-voice.json. … If UDP access is not allowed, TCP port 443 can be used instead, although UDP is preferred.

该小节未点名 `tcr9i.chat.openai.com`。
