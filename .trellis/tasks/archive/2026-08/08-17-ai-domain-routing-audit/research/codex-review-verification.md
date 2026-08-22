# Codex 规划审阅核验（2026-08-17）

对照对象：Codex 对 `08-17-ai-domain-routing-audit` 规划的审阅。本文件只记录核验结果，不替代 PRD。

## 1. suffix 宽于已列 5 个官方主机 — 成立

`design.md` 原方案 `DOMAIN-SUFFIX,chat.openai.com` 的匹配集合是裸域加任意 `*.chat.openai.com`。`prd.md` 证据只列出 5 个主机。Mihomo suffix 语义与仓库 `ruleMatchesHost`（`tests/regression.test.js` 139-140 行：`host === value || host.endsWith("." + value)`）一致。

该差异在原规划中未作为范围取舍写明，与用户「不要过宽」冲突。

`tcr9i.chat.openai.com` 与 Voice 的关联：官方 9247338 把它写在域名 allowlist 里，没有用途说明。同一页的 Voice 小节写的是 UDP 3478 与 `chatgpt-voice.json`，没有点名 `tcr9i`。因此「可能与 Voice 有关」是推测，不是官方事实。规划缺陷是：未记录该主机用途不明。

修订：改为 5 个 `DOMAIN` exact；保留 `tcr9i.chat.openai.com`（官方列表有）；不打开 Livekit / UDP 3478；在产物中写明用途不明。

## 2. DNS 关闭测试断言对象错误 — 成立

`buildNameserverPolicy`（`clash-verge-ai-residential.js` 1330-1331 行）对 suffix 写 `+.${domain}`，对 exact 写裸域名。

2026-08-17 实测默认 policy：

| 探测键 | 是否存在 |
|---|---|
| `chatgpt.com` | 否 |
| `+.chatgpt.com` | 是，值为 `RESIDENTIAL_DOH` |
| `api.openai.com` | 否 |
| `+.api.openai.com` | 是，值为 `RESIDENTIAL_DOH` |
| `oaiusercontent.com` | 否 |
| `+.oaiusercontent.com` | 是，值为 `RESIDENTIAL_DOH` |

`tests/sync-local-config.test.js` 273-279 行对 suffix 主机做 `host in probe.policy`。裸键在默认开启时也不存在，因此该断言在 DNS 未关闭时也会通过。规则侧 `ruleMatchesHost` 仍有效。`implement.md` 原 3.1 只要求追加探测主机，会把伪测试写进新主机。

同文件 712 行「AI DNS policy…」已对 Gemini/Cursor/Grok 使用正确的 `+.` / 裸键，但未断言 OpenAI 核心键。

修订：开关关闭测 `+.chatgpt.com` / `+.api.openai.com` / 五个 exact 裸键；默认开启测这些键等于 `RESIDENTIAL_DOH`；并断言不存在 `+.chat.openai.com`。

## 3. 验收无法证明原生应用结果 — 成立

原 Goal 写「使 ChatGPT 原生应用……进入 AI-家宽」，验收只覆盖合成规则与 `npm run ci`。

`.trellis/spec/frontend/quality-guidelines.md` 55-57 行：宿主 / DNS / 路由变更在可行时应测脱敏真实 Profile；Node 套件不能模拟 Clash 宿主或 Mihomo。README 写明自动化不能替代脱敏 Connections。

修订：自动化验收收窄为「生成配置匹配这 5 个主机」。真实原生应用结果标为 `UNVERIFIED`，除非实现后补脱敏 Connections。

## 4. OpenAI 官方证据不可独立重放 — 成立（已补摘录）

原调研只记 URL 与日期。本机 `web_fetch` 因 Clash fake-ip（`198.18.x`）被 SSRF 拦截；Codex 隔离浏览器遇 Cloudflare 403。

2026-08-17 经 Exa `web_fetch_exa` 再次取到全文。关键摘录见 `openai-9247338-allowlist-excerpt.md`。

## 其它审阅陈述

| 陈述 | 核验 |
|---|---|
| 任务仍是 `planning`，不应 `task.py start` | 成立；`task.json` status=`planning` |
| 本会话 `current-task` 为 none | 成立（`task.py current --source` → none）。任务目录仍在 active 列表 |
| 基线 `npm run ci` 49 项通过 | 采信审阅记录；本轮未重跑 |
| 后续提交须排除 `.gitignore`、`.trellis/.template-hashes.json` | 成立；与本任务无关 |

## 产品决定（修订后）

用户已要求不要过宽。suffix 宽于证据集合。规划改为 5 个 exact 主机。代价：官方以后新增 `*.chat.openai.com` 子域时要再改清单。
