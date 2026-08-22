# 审计并补齐核心 AI 域名路由

## Goal

在现有 6 个产品的默认家宽清单上，只把官方明文列出、当前会漏匹配的 5 个 ChatGPT 主机以 exact 规则写入生成配置，使这些主机在 Mihomo 规则与 nameserver-policy 中指向 `AI-家宽`。

用户价值：官方已列的 ChatGPT 应用主机不再因 `chatgpt.com` suffix 漏匹配而走机场或直连；家宽不吸收未列出的 `*.chat.openai.com` 子域、社区宽规则或其它产品。

真实 ChatGPT 桌面/iOS 客户端是否因此走住宅链路，本任务自动化不能证明，标为 `UNVERIFIED`。

## Background

脚本 v5.7.0 已覆盖 Claude MCP/资产代理、`api.openai.com` 数据驻留前缀、Cursor 细粒度主机、Grok `auth.x.ai` / `api.x.ai`。2026-08-17 对照官方 allowlist 与社区规则集后，官方必列且默认清单未覆盖的只有下列 5 个主机。摘录见 `research/openai-9247338-allowlist-excerpt.md`，审阅核验见 `research/codex-review-verification.md`。

家宽链路是「机场上游 -> 家宽 SOCKS5」。漏规则的结果是该主机走当前 Profile 的机场或 DIRECT。

仓库准入见 `docs/routing-scope.md`：新增域必须有官方资料或脱敏 Connections，并补负向测试。宽泛 provider suffix、市场、更新、下载、媒体、广告、统计、共享基础设施默认不接受。

## Confirmed Facts

- 默认产品集：Claude / Anthropic、ChatGPT / OpenAI / Codex、Gemini Web / AI Studio / Vertex、Antigravity、Cursor、Grok Build。
- 官方 9247338 allowlist 明文列出：`chat.openai.com`、`android.chat.openai.com`、`desktop.chat.openai.com`、`ios.chat.openai.com`、`tcr9i.chat.openai.com`。同一列表还有 `*.openai.com`、`*.oaistatic.com`、`*.auth.openai.com` 等，受仓库准入约束，不注入。
- 现有 `chatgpt.com` suffix 不匹配上述 5 个主机。
- `DOMAIN-SUFFIX,chat.openai.com` 会匹配任意未列出的 `*.chat.openai.com`。用户要求不要过宽，因此不用 suffix。
- `tcr9i.chat.openai.com` 在官方域名表中，无用途说明。官方 Voice 小节写 UDP 3478 与 `chatgpt-voice.json`，未点名该主机。
- suffix 域的 DNS policy 键是 `+.host`，exact 域是裸 `host`。`tests/sync-local-config.test.js` 对 GPT suffix 主机用裸键做 `host in policy`，默认开启时裸键也不存在，该断言不能证明 DNS 已关闭。
- `ROUTE_OPENAI_CORE` 已门控 ChatGPT 产品域。v5.7 删除了 `OPENAI_CORE_EXACT_DOMAINS`，本任务恢复该常量并只放入这 5 个主机。
- 用户 2026-08-17 确认：现有 6 个产品、官方证据、避免过宽。
- Node 测试不能模拟 Clash 宿主或 Mihomo（`.trellis/spec/frontend/quality-guidelines.md`）。

## Requirements

- R1. 产品范围锁定为脚本已声明的 6 个产品。不纳入 Copilot、NotebookLM、Sora、Jules、Windsurf 等社区扩品。
- R2. 恢复 `OPENAI_CORE_EXACT_DOMAINS`，仅含上述 5 个主机。受 `ROUTE_OPENAI_CORE` 门控。不注入 `DOMAIN-SUFFIX,chat.openai.com`。`allPossibleSuffixDomains` 可额外列入 `chat.openai.com`，仅用于清理误注入的 suffix 规则与 `+.chat.openai.com` DNS 键。
- R3. 不新增开关、不改 DNS 架构、不改上游解析、不打开进程兜底 / 公共 DoH / 通用 STUN/TURN。
- R4. 下列主机本轮不注入：`openai.com` / `anthropic.com` / `cursor.sh` 宽后缀、`oaistatic.com`、`chatgpt.livekit.cloud`、ChatGPT Voice UDP 3478、`sora.com`、`chat.com` / `ai.com`、`antigravity-pa.googleapis.com` 及仅社区出现的 Antigravity 主机。
- R5. 正向覆盖 5 个 exact 主机的规则与 DNS 裸键。负向覆盖 `www.openai.com`、`auth.openai.com`、`oaistatic.com`、`oaistatsig.com`、`DOMAIN-SUFFIX,openai.com`、`DOMAIN-SUFFIX,chat.openai.com`（生成输出中不存在）、`+.chat.openai.com`（默认 policy 中不存在）。`routing.openai_core = false` 时 5 个主机的规则与 DNS 键都不在家宽集合中；断言必须使用真实 policy 键（suffix 用 `+.`，exact 用裸名）。
- R6. 托管规则：已有 `DOMAIN,<上述5主机>,AI-家宽` 被清理后按 exact 重注一次。已有 `DOMAIN-SUFFIX,chat.openai.com,AI-家宽` 被清理且不重注。重复执行后规则与 nameserver-policy 幂等。
- R7. 版本升至 5.8.0。`docs/routing-scope.md`、README、CHANGELOG 写明 5 个 exact 与 `UNVERIFIED` 客户端结果。公开模板不写入真实凭据。
- R8. 提交只含本任务路径。不提交 `.gitignore`、`.trellis/.template-hashes.json`、`*.local.toml`、`*.local.js`。

## Acceptance Criteria

- [ ] 默认生成配置含且仅以 exact 形式覆盖这 5 个主机：`DOMAIN,chat.openai.com,AI-家宽` 及 android/desktop/ios/tcr9i 四个子域。各出现一次。
- [ ] 默认 `nameserver-policy` 中这 5 个裸键等于 `RESIDENTIAL_DOH`；不存在 `+.chat.openai.com`；不存在 `DOMAIN-SUFFIX,chat.openai.com` 与 `DOMAIN-SUFFIX,openai.com`。
- [ ] `www.openai.com`、`auth.openai.com`、`oaistatic.com`、`oaistatsig.com` 仍不走家宽。
- [ ] `routing.openai_core = false` 时：5 个主机不被规则匹配；policy 中不存在这 5 个裸键；`+.chatgpt.com`、`+.api.openai.com`、`+.oaiusercontent.com` 也不存在。Claude 等其它核心域不受影响。
- [ ] 输入含 `DOMAIN,chat.openai.com,AI-家宽` 与 `DOMAIN-SUFFIX,chat.openai.com,AI-家宽` 时，前者清理后按 exact 重注一次，后者被清理且不重注。`main` 执行两次后规则与 nameserver-policy 与第一次相同。
- [ ] `SCRIPT_VERSION` 与 `package.json` version 为 `5.8.0`。`npm run ci` 通过。公开模板仍只有 `xxx` 占位符。
- [ ] 文档写明：真实 ChatGPT 原生应用 Connections 结果为 `UNVERIFIED`。若实现后补了脱敏 Connections，再改写该标注。

## Out of Scope

- 社区式全 AI 规则集，或默认打开进程级兜底、公共 DoH/DoT、通用 STUN/TURN。
- `DOMAIN-SUFFIX,chat.openai.com` 以及未列入官方 5 主机的其它 `*.chat.openai.com`。
- ChatGPT Voice（`chatgpt.livekit.cloud`、官方 UDP 3478）及 Sora / 短链跳转域。
- 撤销登录/推理出口分离（`auth.openai.com`、`accounts.google.com`）。
- 为 `downloads.claude.ai` 做后缀内排除（v5.7 已记录取舍）。
- 把 Node 测试结果表述为已在 Clash 宿主或原生应用上验证。
