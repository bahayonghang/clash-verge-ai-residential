# xAI / Grok 官方域名调研

调研日期：2026-08-19。检索工具：Exa（`web_search_exa`）。

## 来源

- https://docs.x.ai/build/enterprise （Enterprise Deployments，含必需/可选主机表）
- https://docs.x.ai/developers/regions （Regional Endpoints）
- https://docs.x.ai/developers/advanced-api-usage/mtls （mTLS Authentication）
- https://docs.x.ai/build/settings/reference （环境变量参考）
- https://www.keysight.com/blogs/en/tech/nwvs/2025/03/28/grok （第三方抓包分析，非官方）

## 官方必需主机（docs.x.ai/build/enterprise 原文）

| Host                      | Purpose                    |
| ------------------------- | -------------------------- |
| `cli-chat-proxy.grok.com` | Inference proxy, settings  |
| `auth.x.ai`               | OAuth2/OIDC authentication |

> If using enterprise OIDC, also allow your IdP's domain (e.g., `login.microsoftonline.com`).

## 官方可选主机（同页原文）

| Host                     | Purpose                                                | Impact if blocked                                                                        |
| ------------------------ | ------------------------------------------------------ | ---------------------------------------------------------------------------------------- |
| `api.x.ai`               | xAI API (direct API-key path)                          | Only needed when using `api_key` auth instead of the inference proxy                     |
| `code.grok.com`          | Remote session sync, sharing, WebSocket relay          | Sessions stay local-only; share links unavailable                                        |
| `assets.grok.com`        | Profile images, UI assets                              | User avatars won't load; no functional impact                                            |
| `x.ai`                   | CLI binary downloads via `curl \| bash` install script | Use `npm install -g @xai-official/grok` as an alternative that doesn't require this host |
| `storage.googleapis.com` | Fallback CDN for CLI binaries                          | Only needed if `x.ai` is unreachable during `curl \| bash` install                       |

> The `x.ai` and `storage.googleapis.com` hosts are only needed for the shell-script installer
> and in-app `grok update`.

## 关键结论

1. **官方只把 `cli-chat-proxy.grok.com` 列为必需的 grok.com 主机**，不是整个 `grok.com` 域。
2. **`assets.grok.com` 官方明确标注 `no functional impact`**，属于头像与 UI 静态资源。
3. `code.grok.com` 承载远程会话同步、分享与 WebSocket 中继，属于会话数据通道。
4. 第三方抓包分析（Keysight，非官方）指出 `grok.com` 承载网页版登录、策略检查与内容加载，
   另有 `auth.grok.com`、`accounts.x.ai` 处理认证，其余主机主要提供静态资源与分析。

## 区域端点与 mTLS（脚本当前会漏掉）

- 区域端点格式：`https://<region>.api.x.ai`，例如 `eu-west-1.api.x.ai`
  （来源：https://docs.x.ai/developers/regions）。
- mTLS 端点：`https://mtls.api.x.ai`，官方描述为「the only change required」
  （来源：https://docs.x.ai/developers/advanced-api-usage/mtls）。

脚本当前使用 `DOMAIN,api.x.ai`（精确匹配），不覆盖 `eu-west-1.api.x.ai` 与 `mtls.api.x.ai`。
使用区域端点或 mTLS 端点的场景下，推理流量会走机场出口。

## 逐条判定

| 脚本条目                 | 规则类型      | 官方出处                                                               | 判定                                                                                                                                      |
| ------------------------ | ------------- | ---------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `grok.com`               | DOMAIN-SUFFIX | 官方仅列 `cli-chat-proxy.grok.com`、`code.grok.com`、`assets.grok.com` | 收窄。网页 Grok 需要 `grok.com` 本身，CLI 需要 `cli-chat-proxy.grok.com` 与 `code.grok.com`；`assets.grok.com` 官方标注无功能影响，可排除 |
| `auth.x.ai`              | DOMAIN        | 官方必需                                                               | 保留                                                                                                                                      |
| `api.x.ai`               | DOMAIN        | 官方可选（API-key 路线）                                               | 保留，但建议改为 `DOMAIN-SUFFIX,api.x.ai` 以覆盖区域端点与 `mtls.api.x.ai`                                                                |
| `x.ai`（安装脚本）       | 未注入        | 官方标注仅安装用                                                       | 正确排除                                                                                                                                  |
| `storage.googleapis.com` | 未注入        | 官方标注仅安装回退 CDN                                                 | 正确排除                                                                                                                                  |
| `api.mixpanel.com`       | 未注入        | 未在官方企业文档出现                                                   | 正确排除                                                                                                                                  |

## 缺失项

| 主机                            | 官方用途                             | 缺失影响                                                 |
| ------------------------------- | ------------------------------------ | -------------------------------------------------------- |
| `eu-west-1.api.x.ai` 等区域端点 | 指定区域处理请求                     | 使用区域端点时推理流量不走家宽                           |
| `mtls.api.x.ai`                 | 企业 mTLS 推理端点                   | 使用 mTLS 时推理流量不走家宽                             |
| `auth.grok.com`                 | 第三方分析指出用于认证（无官方出处） | 存疑。当前被 `DOMAIN-SUFFIX,grok.com` 覆盖；若收窄需确认 |
| `accounts.x.ai`                 | 第三方分析指出用于认证（无官方出处） | 存疑。当前未覆盖                                         |

## 过度覆盖实测

`DOMAIN-SUFFIX,grok.com` 把该域下全部子域纳入家宽。官方文档中该域下明确的非必需主机是
`assets.grok.com`（头像与 UI 资源，官方标注无功能影响）。
