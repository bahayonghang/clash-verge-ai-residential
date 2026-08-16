# AI 服务端点域名核对（2026-08，官方文档背书）

## Anthropic（官方网络要求：code.claude.com/docs/en/network-config.md）

| 域名 | 官方定位 | 脚本现状 | 处置 |
|---|---|---|---|
| api.anthropic.com | API + feature flag + 遥测 | exact ✓ | 保留 |
| claude.ai / claude.com / *.claudeusercontent.com | 产品/登录/制品 | suffix ✓ | 保留（platform.claude.com 因此已被覆盖） |
| platform.claude.com | Console 认证 + claude.ai OAuth token 交换/刷新 | 经 claude.com suffix 已覆盖 | 无需新增 |
| mcp-proxy.anthropic.com | claude.ai MCP connector 代理（官方必列） | **缺失** | 新增 exact |
| assets-proxy.anthropic.com | 桌面/网页资产代理（官方警告缺失会白屏） | **缺失** | 新增 exact |
| bridge.claudeusercontent.com | Claude in Chrome WS 桥 | 经 suffix 已覆盖 | 无需新增 |
| downloads.claude.ai | 安装器/更新 | 经 claude.ai suffix 走家宽 | 记录取舍，不拆分（见 design） |
| statsig.anthropic.com / sentry.io | 官方网络表未列（flag 走 api.anthropic.com） | 未纳入 ✓ | 不加 |
| 160.79.104.0/23、2607:6bc0::/48 | 官方入站段（platform.claude.com/docs/en/api/ip-addresses） | ✓ | 保留 |

## OpenAI / Codex（官方 allowlist：help.openai.com 9247338；Codex 文档：learn.chatgpt.com）

| 域名 | 官方定位 | 脚本现状 | 处置 |
|---|---|---|---|
| chatgpt.com（含 wss、backend-api/codex） | Codex CLI 默认推理路线 | suffix ✓ | 保留 |
| api.openai.com | API-key 路线 | exact | **改 suffix** |
| us.api.openai.com / eu.api.openai.com | Codex 数据驻留前缀（官方 config 文档） | **exact 不匹配 → 缺失** | 随上条修复 |
| oaiusercontent.com | 上传/制品 | suffix ✓ | 保留 |
| oaistatsig.com | 官方遥测（可选） | 未纳入 ✓ | 不加 |
| auth.openai.com | 认证 | 有意排除（登录/推理分离，文档化） | 保持 |

## Cursor（官方：cursor.com/docs/enterprise/network-configuration）

现状与官方清单吻合（api2/3/4/5、repo42、authenticate/authenticator/authentication、
gcpp、cursorvm.com 均已覆盖；marketplace/cursor-cdn/downloads/S3 正确排除）。
官方未列 api6 与 api*.cursorapi.com —— 维持现状，不加。

## xAI Grok（官方：docs.x.ai/build/enterprise）

| 域名 | 官方定位 | 脚本现状 | 处置 |
|---|---|---|---|
| cli-chat-proxy.grok.com | CLI 推理 + 设置（must-allow） | grok.com suffix ✓ | 保留 |
| code.grok.com / assets.grok.com | 会话同步 / UI 资源（可选） | grok.com suffix ✓ 覆盖 | 保留 |
| auth.x.ai | OAuth2/OIDC 认证（must-allow） | **缺失** | 新增 exact |
| api.x.ai | API-key 直连推理端点（官方可选；API base = api.x.ai/v1） | **缺失** | 新增 exact |
| x.ai | 安装脚本/隐私端点 | 正确排除 ✓ | 保持 |

## Google Gemini / Antigravity

cloudcode-pa.googleapis.com 有 gemini-cli 源码级证据；clients6 系为社区观察但实践必需；
Antigravity 官方无 allowlist（企业页仅 aiplatform.googleapis.com）。现状覆盖足够，不加。

## 来源

- https://code.claude.com/docs/en/network-config.md
- https://platform.claude.com/docs/en/api/ip-addresses
- https://help.openai.com/en/articles/9247338-network-recommendations-for-chatgpt-errors-on-web-and-apps
- https://learn.chatgpt.com/docs/codex/cli 、https://learn.chatgpt.com/docs/config-file/config-reference
- https://cursor.com/docs/enterprise/network-configuration
- https://docs.x.ai/build/enterprise 、https://docs.x.ai/docs
