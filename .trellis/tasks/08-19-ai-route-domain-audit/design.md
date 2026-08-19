# 技术设计：AI 家宽路由域名清单收窄

## 1. 判定口径

用户约束是「只有网页 Chat、本机 CLI、Cursor 等客户端连接官方的流量走家宽」。
把它拆成两类载体，因为两类的收窄代价不同。

### 载体 A：浏览器内的网页 Chat

`claude.ai`、`chatgpt.com`、`grok.com`、`gemini.google.com`、`aistudio.google.com`。
同一个浏览器会话会并发访问同一产品的多个子域，并携带同一套 Cookie。
把其中一部分子域改走机场出口，会让服务端在**同一个已登录会话内**观察到两个出口 IP。
`docs/routing-scope.md` 的 Authentication exit split 一节已记录这种分裂对风控的影响。

因此对载体 A，**产品顶级域保持后缀匹配**。多代理的是状态页与静态资源的带宽，
换取的是同一会话内出口 IP 一致。收窄这类子域属于用带宽换风控风险，不划算。

### 载体 B：本机 CLI 与桌面/IDE 客户端

Claude Code、Codex、Grok CLI、Cursor、Antigravity。
这类进程的非推理请求（文档站、更新下载、扩展市场、静态资源）不携带推理会话凭据，
与推理连接是独立的 TCP 会话。改走机场出口不产生会话内 IP 分裂。

因此对载体 B，**按官方文档逐条收窄到必需主机**。

## 2. 逐条判定表（全部 49 条注入规则）

### 2.1 保留（有官方出处，且符合范围）

| 规则                                                                    | 官方出处                                                    | 保留理由                                                                                       |
| ----------------------------------------------------------------------- | ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| `DOMAIN-SUFFIX,claude.ai`                                               | code.claude.com/docs/en/network-config                      | 账户认证必需；载体 A                                                                           |
| `DOMAIN-SUFFIX,claude.com`                                              | 同上                                                        | `platform.claude.com` 承担 OAuth token 交换/刷新/吊销，只有后缀能覆盖；收窄为 exact 会破坏登录 |
| `DOMAIN,api.anthropic.com`                                              | 同上                                                        | 模型推理主通道                                                                                 |
| `DOMAIN,mcp-proxy.anthropic.com`                                        | 同上                                                        | claude.ai MCP connector 代理                                                                   |
| `IP-CIDR,160.79.104.0/23` / `IP-CIDR6,2607:6bc0::/48`                   | platform.claude.com/docs/en/api/ip-addresses                | 官方 inbound 段，即客户端连入地址，兜底方向正确                                                |
| `DOMAIN-SUFFIX,chatgpt.com`                                             | help.openai.com/9247338 官方 `*.chatgpt.com`                | 与官方通配一致；Codex WebSocket 也在此域                                                       |
| `DOMAIN-SUFFIX,api.openai.com`                                          | 官方 `*.openai.com`                                         | 覆盖 `us.` / `eu.` 数据驻留前缀                                                                |
| `DOMAIN-SUFFIX,oaiusercontent.com`                                      | 官方 `*.oaiusercontent.com`                                 | 用户上传与生成内容                                                                             |
| `DOMAIN,chat.openai.com` 等五个                                         | 官方逐条列出                                                | 官方清单成员；`tcr9i.` 官方无用途说明                                                          |
| `DOMAIN,auth.x.ai`                                                      | docs.x.ai/build/enterprise                                  | 官方标注必需                                                                                   |
| `DOMAIN-SUFFIX,api5.cursor.sh`                                          | cursor.com/docs/enterprise/network-configuration            | 官方列出六个 `agent*` 子域，必须后缀                                                           |
| `DOMAIN-SUFFIX,gcpp.cursor.sh`                                          | 同上                                                        | 官方主机为 `us-asia.` / `us-eu.` / `us-only.` 前缀，必须后缀                                   |
| `DOMAIN-SUFFIX,authentication.cursor.sh`                                | 同上                                                        | 官方另列 `prod.authentication.cursor.sh`，必须后缀                                             |
| `DOMAIN-SUFFIX,cursorvm.com`                                            | 官方 `*.cursorvm.com` / `*.*.cursorvm.com`                  | 与官方通配一致                                                                                 |
| `DOMAIN,api3.cursor.sh` / `DOMAIN,api4.cursor.sh`                       | 同上                                                        | Cursor Tab                                                                                     |
| `DOMAIN,authenticator.cursor.sh`                                        | 同上                                                        | 登录 UI 与 webview                                                                             |
| `DOMAIN,cloudcode-pa.googleapis.com`                                    | docs.cloud.google.com/gemini/docs/codeassist/network-access | 官方唯一确认的 Code Assist 端点，且官方要求不用通配                                            |
| `DOMAIN,generativelanguage.googleapis.com`                              | Gemini Developer API 公开端点                               | 推理端点                                                                                       |
| `DOMAIN-SUFFIX,gemini.google.com` / `DOMAIN-SUFFIX,aistudio.google.com` | 产品入口                                                    | 载体 A                                                                                         |

小计 26 条。

### 2.2 收窄

| 当前规则                                                | 问题                                                               | 建议                                                                                                                                | 依据                                                    |
| ------------------------------------------------------- | ------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| `DOMAIN-SUFFIX,grok.com`                                | 覆盖 `assets.grok.com`，官方标注 `no functional impact`            | 载体 A 部分保留 `grok.com` 后缀；若启用严格模式，改为 `DOMAIN,grok.com` + `DOMAIN,cli-chat-proxy.grok.com` + `DOMAIN,code.grok.com` | docs.x.ai/build/enterprise 必需/可选主机表              |
| `DOMAIN-SUFFIX,antigravity.google`                      | 覆盖 `docs.antigravity.google`、`download.antigravity.google`      | 收窄为 `DOMAIN,antigravity.google`，把 docs/download 排除                                                                           | 载体 B；官方无防火墙清单，按用途分类                    |
| `DOMAIN-SUFFIX,api2.cursor.sh`                          | 官方只列该精确主机                                                 | 改为 `DOMAIN,api2.cursor.sh`                                                                                                        | cursor.com 精确主机清单                                 |
| `DOMAIN-SUFFIX,authenticate.cursor.sh`                  | 官方只列该精确主机                                                 | 改为 `DOMAIN,authenticate.cursor.sh`                                                                                                | 同上                                                    |
| `DOMAIN-REGEX,^adminportal[0-9]+\.cursor\.sh$`          | 官方只有 `adminportal42.cursor.sh`，匹配 0-999 全部编号            | 改为 `DOMAIN,adminportal42.cursor.sh`，或保留正则并在文档中维持「前向兼容策略，非官方通配」标注                                     | 同上                                                    |
| `DOMAIN-REGEX,^[a-z0-9-]+-aiplatform\.googleapis\.com$` | 覆盖全部 Vertex AI 区域端点，包含与 Gemini 无关的 GCP 机器学习流量 | 保留匹配能力，但把它移到独立开关（默认关），或把区域段收窄为官方区域名枚举                                                          | cloud.google.com/vertex-ai/docs/reference/rest 端点全表 |

小计 6 条。

### 2.3 补充

| 建议新增                                               | 官方出处                                                                   | 缺失影响                                                                                                                |
| ------------------------------------------------------ | -------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `DOMAIN-SUFFIX,api.x.ai`（替换现有 `DOMAIN,api.x.ai`） | docs.x.ai/developers/regions、docs.x.ai/developers/advanced-api-usage/mtls | 现有精确匹配漏掉 `<region>.api.x.ai`（如 `eu-west-1.api.x.ai`）与 `mtls.api.x.ai`；使用区域或 mTLS 端点时推理流量走机场 |
| `DOMAIN,code.grok.com`                                 | docs.x.ai/build/enterprise                                                 | 当前由 `grok.com` 后缀覆盖；若执行 grok 收窄，必须显式补回，否则远程会话同步与分享失效                                  |

### 2.4 存疑（无官方出处，建议标注而非立即删除）

| 规则                                               | 状态                                                          |
| -------------------------------------------------- | ------------------------------------------------------------- |
| `DOMAIN-SUFFIX,clau.de`                            | 仓库初始提交引入，无依据记录。疑为短链域                      |
| `DOMAIN-SUFFIX,claudemcpclient.com`                | 同上                                                          |
| `DOMAIN-SUFFIX,claudemcpcontent.com`               | 同上                                                          |
| `DOMAIN-SUFFIX,claudeusercontent.com`              | 同上                                                          |
| `DOMAIN,a-api.anthropic.com`                       | 本次检索未找到官方说明                                        |
| `DOMAIN,assets-proxy.anthropic.com`                | CHANGELOG v5.7 记录来源为官方网络配置文档，本次检索未复现该行 |
| `DOMAIN,api.cursor.com`                            | 未出现在 Cursor 官方网络配置文档                              |
| `DOMAIN,daily-cloudcode-pa.googleapis.com`         | 无官方出处，命名指向每日构建端点                              |
| `DOMAIN,cloudaicompanion.googleapis.com`           | 无官方防火墙出处                                              |
| `DOMAIN,geminicloudassist.googleapis.com`          | 无官方防火墙出处                                              |
| `DOMAIN,alkalicore-pa.clients6.google.com` 等三个  | 无官方出处                                                    |
| `DOMAIN,aiplatform.us.rep.googleapis.com` / `.eu.` | 数据驻留概念有官方出处，主机名逐字出处未找到                  |
| `DOMAIN,aiplatform.googleapis.com`                 | 官方端点，但属于通用云 API，超出用户设定的三类载体            |

小计 17 条。

## 3. 兼容性：收窄后的旧规则清理

`buildManagedRuleSet()` 依据 `allPossibleSuffixDomains()` / `allPossibleExactDomains()` /
`allPossibleDomainRegexes()` 构造清理集合，与激活开关无关。
`buildManagedDnsPolicyKeySet()` 用同一组清单构造 DNS 键。

已实测验证：把条目留在 allPossible 清单但移出激活清单后，旧规则被幂等清除，
未知用户规则原样保留。

```
输入：DOMAIN-SUFFIX,chat.openai.com,AI-家宽 / DOMAIN,api.openai.com,AI-家宽
      DOMAIN-SUFFIX,claude.com,AI-家宽 / DOMAIN-SUFFIX,example-user-rule.com,AI-家宽
输出：DOMAIN-SUFFIX,example-user-rule.com,AI-家宽
```

因此任何收窄都必须遵守一条约束：**被移除的域名必须保留在 `allPossible*` 清单中**，
否则升级用户配置里的旧规则不会被清理，会长期残留。

## 4. DNS 与规则的不对称

`buildNameserverPolicy()` 只遍历 `activeSuffixDomains()` 与 `activeExactDomains()`，
不遍历 `activeDomainRegexes()`。当前两条正则匹配的主机
（Vertex AI 区域端点、`adminportal<N>.cursor.sh`）路由走家宽，DNS 解析走机场 DoH。

在 `enhanced-mode: fake-ip` 下客户端拿到 fake IP，真实解析在出口侧完成，
该不对称不改变最终出口 IP。影响范围限于 DNS 查询本身经由机场链路。
本次审计记录该现象，不作为必须修复项。

## 5. 建议的开关结构

收窄不应写死，因为载体 A 的取舍随用户风控敏感度变化。建议新增两个开关：

| 开关                         | 默认   | 作用                                                                                                                       |
| ---------------------------- | ------ | -------------------------------------------------------------------------------------------------------------------------- |
| `routing.grok_web_assets`    | `true` | `true` 保持 `DOMAIN-SUFFIX,grok.com`；`false` 收窄为 `grok.com` + `cli-chat-proxy.grok.com` + `code.grok.com` 三条精确规则 |
| `routing.vertex_ai_regional` | `true` | `true` 保持区域正则；`false` 只保留 `aiplatform.googleapis.com` 全局端点                                                   |

开关经 `scripts/sync-local-config.js` 的 `routing` 表接线，
并在 `clash-verge-ai-residential.local.toml.example` 中补齐同名键。

## 6. 风险与取舍

1. 收窄载体 A 的子域会在同一登录会话内产生两个出口 IP。本设计因此不收窄
   `claude.ai` / `claude.com` / `chatgpt.com` / `gemini.google.com` / `aistudio.google.com`，
   只把 `grok.com` 的收窄放在默认关闭的开关后。
2. 删除存疑条目会失去当前可能生效的覆盖，且四个 Claude 相关域（`clau.de`、
   `claudemcpclient.com`、`claudemcpcontent.com`、`claudeusercontent.com`）
   的真实用途未查明。建议本次只标注，不删除。
3. `api.x.ai` 由 exact 改 suffix 会扩大匹配范围到 `*.api.x.ai`。该命名空间由 xAI 控制，
   官方已列出区域端点与 mTLS 端点两类子域，扩大范围有官方依据。
