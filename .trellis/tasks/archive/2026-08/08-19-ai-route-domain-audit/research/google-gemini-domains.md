# Google Gemini / AI Studio / Antigravity 官方域名调研

调研日期：2026-08-19（含同日二次检索更正）。检索工具：Exa。

## 来源

- https://antigravity.google/docs/enterprise/ （Antigravity Enterprise，含区域端点矩阵）
- https://docs.cloud.google.com/gemini-enterprise-agent-platform/resources/locations （Deployments and endpoints，含多区域主机名表）
- https://docs.cloud.google.com/gemini-enterprise-agent-platform/models/partner-models/use-partner-models
- https://cloud.google.com/gemini/docs/discover/set-up-gemini （Gemini Code Assist 防火墙配置）
- https://developers.google.com/gemini-code-assist/docs/set-up-gemini-standard-enterprise （同上镜像）
- https://docs.cloud.google.com/gemini/docs/codeassist/network-access
- https://cloud.google.com/vertex-ai/docs/reference/rest （Vertex AI 服务端点全表）
- https://antigravity.google/docs/ide/allowlist-denylist （Agent 浏览器 URL 允许列表，非网络放行清单）
- https://github.com/google-gemini/gemini-cli/issues/4552 （社区请求 CLI 防火墙清单，未关闭）

## 官方防火墙清单（Gemini Code Assist，原文）

> In addition to enabling the Gemini for Google Cloud API, users behind firewalls also need to
> allow traffic to pass through for the following APIs:
>
> - `oauth2.googleapis.com`: used to sign in to Google Cloud.
> - `serviceusage.googleapis.com`: used for checking that the user's Gemini Code Assist project is properly configured.
> - `cloudaicompanion.googleapis.com`: the primary Gemini Code Assist API endpoint.
> - `cloudcode-pa.googleapis.com`: an internal API that provides IDE-related features.
> - `cloudresourcemanager.googleapis.com`: used in the IDEs for project pickers.
> - `people.googleapis.com`: provides access to information about profiles and contacts.

结论：`cloudaicompanion.googleapis.com` 与 `cloudcode-pa.googleapis.com` 都有官方出处，
且 `cloudaicompanion` 被标注为**主要 API 端点**。清单同时包含
`oauth2` / `serviceusage` / `cloudresourcemanager` / `people`，脚本以默认关闭的开关排除这四项，
属于有意的范围收窄，不是遗漏。

`docs.cloud.google.com/gemini/docs/codeassist/network-access` 另有一条约束：

> Configure the proxy to intercept all outgoing requests to the Gemini Code Assist endpoint
> (`https://cloudcode-pa.googleapis.com`). Don't use wildcards (`*`) when you specify the
> Gemini Code Assist endpoint.

脚本对该主机使用 `DOMAIN` 精确匹配，与官方口径一致。

## 官方多区域端点主机名（原文）

> ### Multi-region endpoints
> Multi-region endpoints allow you to ensure that machine learning processing of Customer Data
> by the service stays within a specific jurisdictional boundary, such as the United States or
> the European Union.
>
> The following table lists the hostnames for multi-region endpoints:
>
> | Multi-region | Location | Hostname |
> |---|---|---|
> | United States | `us` | `https://aiplatform.us.rep.googleapis.com` |
> | European Union | `eu` | `https://aiplatform.eu.rep.googleapis.com` |

来源：https://docs.cloud.google.com/gemini-enterprise-agent-platform/resources/locations

partner-models 页复述同一组主机名，并给出 curl 示例
`https://aiplatform.us.rep.googleapis.com/v1/projects/.../publishers/anthropic/models/...`。

## Antigravity 企业部署（原文）

> Enable the Agent Platform API: Enable the Agent Platform API (`aiplatform.googleapis.com`)
> to allow Antigravity clients to connect to your project's model endpoints.
>
> ## Regional Endpoints & Capability Matrix
> Antigravity CLI and Antigravity 2.0 support multi-region deployment endpoints to satisfy
> regional data residency requirements:
>
> | Endpoint Region | Base Endpoint URI | Supported Capabilities |
> |---|---|---|
> | Global | `global` | Text Generation, Code Inference, Multimodal, Image Generation |
> | US Multi-Region | `us` | Text Generation, Code Inference, Multimodal |
> | EU Multi-Region | `eu` | Text Generation, Code Inference, Multimodal |

来源：https://antigravity.google/docs/enterprise/

**这条证据改变了 Vertex AI 相关规则的定性。** `aiplatform.googleapis.com` 与两个
`.rep.` 多区域主机不是「与 AI 无关的通用云 API」，而是 Antigravity 企业部署下
Antigravity CLI 与 Antigravity 2.0 的推理端点。把它们默认关闭会切断 Antigravity 企业版的推理链路。

## Vertex AI 区域端点

`cloud.google.com/vertex-ai/docs/reference/rest` 列出 30+ 个
`<region>-aiplatform.googleapis.com` 主机，region 为 Google Cloud 区域名。
`REGION-aiplatform.googleapis.com` 是官方描述的标准访问方式。
Gemini CLI 在 Vertex AI 认证模式下使用该形式（gemini-cli issue #4552 中用户实测
`europe-west4-aiplatform.googleapis.com`）。

## Antigravity 没有客户端防火墙放行清单

`antigravity.google/docs/ide/allowlist-denylist` 与 `antigravity.google/docs/permissions`
描述的是 Agent 浏览器子代理可访问哪些 URL、以及沙箱出站允许域，属于产品内权限模型。
Antigravity 客户端自身连接后端所需的主机清单，官方只在 Enterprise 页以
「Agent Platform API + global/us/eu 端点」的形式给出，没有逐条主机表。

## 逐条判定（二次检索后更正）

| 脚本条目 | 规则类型 | 官方出处 | 判定 |
|---|---|---|---|
| `cloudcode-pa.googleapis.com` | DOMAIN | Code Assist 防火墙清单 | 保留。官方要求精确匹配 |
| `cloudaicompanion.googleapis.com` | DOMAIN | Code Assist 防火墙清单 | **保留（更正）**。官方标注为 primary Gemini Code Assist API endpoint |
| `generativelanguage.googleapis.com` | DOMAIN | Gemini Developer API 端点 | 保留 |
| `aiplatform.googleapis.com` | DOMAIN | Antigravity Enterprise + Vertex AI 全局端点 | **保留（更正定性）**。Antigravity 企业部署的推理端点，不是无关的通用云 API |
| `aiplatform.us.rep.googleapis.com` | DOMAIN | Deployments and endpoints 多区域主机名表 | **保留（更正）**。官方逐字列出 |
| `aiplatform.eu.rep.googleapis.com` | DOMAIN | 同上 | **保留（更正）**。官方逐字列出 |
| `^[a-z0-9-]+-aiplatform\.googleapis\.com$` | DOMAIN-REGEX | Vertex AI 端点全表 | 保留。覆盖官方全部区域端点；正则比官方区域名集合宽，但该命名空间由 Google 控制 |
| `gemini.google.com` | DOMAIN-SUFFIX | 产品入口，无防火墙文档 | 保留。网页 Chat，属载体 A |
| `aistudio.google.com` | DOMAIN-SUFFIX | 产品入口，无防火墙文档 | 保留。网页产品，属载体 A |
| `antigravity.google` | DOMAIN-SUFFIX | 无防火墙清单 | 收窄为 `DOMAIN,antigravity.google`。后缀命中 `docs.` 与 `download.` 子域 |
| `daily-cloudcode-pa.googleapis.com` | DOMAIN | 无官方出处 | 退出激活清单。命名指向每日构建/预发布端点 |
| `geminicloudassist.googleapis.com` | DOMAIN | 无官方出处 | 退出激活清单。该 API 存在于 GCP 目录，但不在 Code Assist 防火墙清单中 |
| `alkalicore-pa.clients6.google.com` | DOMAIN | 无官方防火墙清单 | 保留，标 UNVERIFIED。用户确认，避免 AI Studio 网页出口分裂 |
| `alkalimakersuite-pa.clients6.google.com` | DOMAIN | 无官方防火墙清单 | 保留，标 UNVERIFIED。社区标为 GenerateContent 主机 |
| `webchannel-alkalimakersuite-pa.clients6.google.com` | DOMAIN | 无官方防火墙清单 | 保留，标 UNVERIFIED。AI Studio 流式通道 |
| `accounts.google.com`、`oauth2.googleapis.com` | 未注入（开关默认关） | 在 Code Assist 官方清单中 | 有意排除，与 Authentication exit split 取舍一致 |
| `serviceusage` / `cloudresourcemanager` / `iam` / `apihub` | 未注入（开关默认关） | 前两者在官方清单中 | 有意排除，项目配置类 |
| `update.googleapis.com`、`dl.google.com`、`open-vsx.org`、firebase/feedback 遥测 | 未注入（开关默认关） | 更新与遥测 | 正确排除 |

## 更正记录

本文件第一版把 `aiplatform.us.rep.googleapis.com` 与 `aiplatform.eu.rep.googleapis.com`
判为「主机名逐字出处未找到」，把 `cloudaicompanion.googleapis.com` 判为「无官方防火墙出处」，
并建议把 Vertex AI 规则移到默认关闭的开关后。三项判定均已更正：

- 两个 `.rep.` 主机名在 Deployments and endpoints 页有逐字表格。
- `cloudaicompanion.googleapis.com` 在 Gemini Code Assist 防火墙清单中标注为主要端点。
- Antigravity Enterprise 页确认 `aiplatform.googleapis.com` 与 global/us/eu 端点是
  Antigravity CLI / 2.0 的推理路径，因此默认关闭会造成功能中断。

## 过度覆盖实测

`DOMAIN-SUFFIX,antigravity.google` 命中 `www.antigravity.google`、`docs.antigravity.google`、
`download.antigravity.google`。这是本产品线唯一确认的过度覆盖点。

`DOMAIN-SUFFIX,gemini.google.com` 与 `DOMAIN-SUFFIX,aistudio.google.com`
命中其下全部子域；这两个域本身是单一产品主机，目前没有已知的非 AI 子域。

## DNS 与规则的不对称

`buildNameserverPolicy` 只为 `activeSuffixDomains()` 与 `activeExactDomains()` 生成键，
不处理 `activeDomainRegexes()`。因此 `us-central1-aiplatform.googleapis.com` 与
`adminportal42.cursor.sh` 的路由走家宽，DNS 解析走机场 DoH。

在 `enhanced-mode: fake-ip` 下客户端拿到 fake IP，真实解析在出口侧完成，
该不对称对最终出口 IP 无影响；影响范围限于 DNS 查询本身经由机场链路。

## 更正记录（2026-08-19 第三次检索）

**三条 `alkali*` 用户确认保留并标 UNVERIFIED。**

仍无官方防火墙清单。Google 开发者论坛与社区抓包把
`alkalimakersuite-pa.clients6.google.com` 标为 AI Studio 网页
`MakerSuiteService/GenerateContent` RPC 主机。
三条主机留在 `GEMINI_WEB_EXACT_DOMAINS`。部署后用脱敏 Connections 补证。
