# Google Gemini / AI Studio / Antigravity 官方域名调研

调研日期：2026-08-19。检索工具：Exa（`web_search_exa`）。

## 来源

- https://docs.cloud.google.com/gemini/docs/codeassist/network-access （Gemini Code Assist 网络访问）
- https://cloud.google.com/vertex-ai/docs/general/googleapi-access-methods （Vertex AI API 访问方式）
- https://cloud.google.com/vertex-ai/docs/reference/rest （Vertex AI 服务端点全表）
- https://docs.cloud.google.com/gemini-enterprise-agent-platform/resources/data-residency （数据驻留）
- https://antigravity.google/docs/ide/allowlist-denylist （Antigravity 浏览器 URL 允许/拒绝列表）
- https://github.com/google-gemini/gemini-cli/issues/4552 （社区请求官方防火墙清单，未关闭）
- https://antigravitylab.net/en/articles/tips/antigravity-corporate-proxy-firewall-connection-fix （第三方，非官方）

## 关键结论：Antigravity 没有官方防火墙放行清单

检索未找到 Google 发布的 Antigravity IDE 网络放行文档。
`antigravity.google/docs/ide/allowlist-denylist` 与 `antigravity.google/docs/permissions`
描述的是 **Agent 浏览器子代理可访问哪些 URL** 与 **沙箱出站允许域**，
属于产品内的权限模型，不是客户端自身连接后端所需的防火墙清单。

Gemini CLI 侧存在公开 issue（google-gemini/gemini-cli#4552）请求提供与 Gemini Code Assist
同级的 URL 清单，说明 Google 未对 CLI 发布该清单。

因此脚本中 Antigravity 与 Gemini Web 的多数条目**没有官方防火墙文档背书**。

## 官方确认的端点

### Gemini Code Assist

> Configure the proxy to intercept all outgoing requests to the Gemini Code Assist endpoint
> (`https://cloudcode-pa.googleapis.com`). Don't use wildcards (`*`) when you specify the
> Gemini Code Assist endpoint.

来源：https://docs.cloud.google.com/gemini/docs/codeassist/network-access

结论：`cloudcode-pa.googleapis.com` 官方确认，且官方明确要求**不要使用通配**。
脚本使用 `DOMAIN` 精确匹配，与官方口径一致。

### Vertex AI 端点

> - Global endpoints: These endpoints (like `https://aiplatform.googleapis.com`) don't specify
>   a region in the hostname.
> - The regional Gemini Enterprise Agent Platform endpoint (`REGION-aiplatform.googleapis.com`)
>   is the standard way to access Google APIs.

官方端点全表列出 30+ 个 `<region>-aiplatform.googleapis.com` 主机，region 取值为
Google Cloud 区域名（`us-central1`、`europe-west4`、`asia-northeast1` 等）。

来源：https://cloud.google.com/vertex-ai/docs/reference/rest

结论：脚本的 `^[a-z0-9-]+-aiplatform\.googleapis\.com$` 覆盖全部官方区域端点。
该正则比官方区域名集合宽，但 `*-aiplatform.googleapis.com` 命名空间由 Google 控制，
实际不存在非 Vertex AI 的主机。**风险是范围问题而非匹配问题**：Vertex AI 是通用云 API，
任何 GCP 机器学习工作负载都会命中，不限于 Gemini CLI 或 Antigravity。

### 数据驻留端点

官方文档描述 jurisdictional multi-region endpoints（美国 / 欧盟司法辖区）。
脚本中的 `aiplatform.us.rep.googleapis.com` / `aiplatform.eu.rep.googleapis.com`
与该概念对应，但本次检索**未找到逐字列出这两个主机名的官方页面**。

## 逐条判定

| 脚本条目                                                                         | 规则类型             | 官方出处                                       | 判定                                                                            |
| -------------------------------------------------------------------------------- | -------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------- |
| `cloudcode-pa.googleapis.com`                                                    | DOMAIN               | Gemini Code Assist 网络访问文档                | 保留。官方唯一确认的 Code Assist 端点                                           |
| `generativelanguage.googleapis.com`                                              | DOMAIN               | Gemini Developer API 公开端点                  | 保留。Gemini API 推理端点                                                       |
| `aiplatform.googleapis.com`                                                      | DOMAIN               | Vertex AI 全局端点                             | 保留，但属于通用云 API，超出「网页 Chat / 本机 CLI / 客户端」范围               |
| `^[a-z0-9-]+-aiplatform\.googleapis\.com$`                                       | DOMAIN-REGEX         | Vertex AI 区域端点全表                         | 保留匹配能力；建议将区域段收窄为官方区域名，或明确记录其覆盖通用 Vertex AI 流量 |
| `aiplatform.us.rep.googleapis.com`、`aiplatform.eu.rep.googleapis.com`           | DOMAIN               | 概念有官方出处，主机名逐字出处未找到           | 存疑                                                                            |
| `daily-cloudcode-pa.googleapis.com`                                              | DOMAIN               | 无官方出处                                     | 存疑。命名形式指向每日构建/预发布端点，普通用户不会访问                         |
| `cloudaicompanion.googleapis.com`                                                | DOMAIN               | 无官方防火墙出处（该 API 存在于 GCP API 目录） | 存疑                                                                            |
| `geminicloudassist.googleapis.com`                                               | DOMAIN               | 无官方防火墙出处（该 API 存在于 GCP API 目录） | 存疑                                                                            |
| `gemini.google.com`                                                              | DOMAIN-SUFFIX        | 产品入口，无防火墙文档                         | 保留。网页 Chat 在用户设定范围内                                                |
| `aistudio.google.com`                                                            | DOMAIN-SUFFIX        | 产品入口，无防火墙文档                         | 保留。网页产品在用户设定范围内                                                  |
| `alkalicore-pa.clients6.google.com` 等三个                                       | DOMAIN               | 无官方出处                                     | 存疑。命名符合 Google 内部 `-pa` 后端约定，但无公开文档                         |
| `antigravity.google`                                                             | DOMAIN-SUFFIX        | 无官方防火墙出处                               | 收窄。后缀覆盖 `docs.` 与 `download.` 子域                                      |
| `accounts.google.com`、`oauth2.googleapis.com`                                   | 未注入（开关默认关） | 第三方指南列为必需                             | 有意排除。与 Authentication exit split 取舍一致                                 |
| Service Usage / Resource Manager / IAM / API Hub                                 | 未注入（开关默认关） | 项目配置类 API                                 | 正确排除                                                                        |
| `update.googleapis.com`、`dl.google.com`、`open-vsx.org`、firebase/feedback 遥测 | 未注入（开关默认关） | 更新与遥测                                     | 正确排除                                                                        |

## 过度覆盖实测

`DOMAIN-SUFFIX,antigravity.google` 命中 `www.antigravity.google`、`docs.antigravity.google`、
`download.antigravity.google`。
`DOMAIN-SUFFIX,gemini.google.com` 与 `DOMAIN-SUFFIX,aistudio.google.com` 命中其下全部子域；
这两个域本身就是单一产品主机，后缀匹配的额外范围目前没有已知的非 AI 主机。

## DNS 与规则的不对称

`buildNameserverPolicy` 只为 `activeSuffixDomains()` 与 `activeExactDomains()` 生成键，
不处理 `activeDomainRegexes()`。因此：

- `us-central1-aiplatform.googleapis.com` 与 `adminportal42.cursor.sh` 的**路由**走家宽，
  但**DNS 解析**走机场 DoH（`buildUpstreamDoh` 生成的 nameserver）。
- 在 `enhanced-mode: fake-ip` 下，客户端拿到的是 fake IP，真实解析在出口侧完成，
  该不对称对最终出口 IP 无影响；影响范围是 DNS 查询本身经由机场链路。
