# Cursor 官方域名调研

调研日期：2026-08-19。检索工具：Exa（`web_search_exa`）。

## 来源

- https://cursor.com/docs/enterprise/network-configuration （Network Configuration）
- https://cursor.com/help/troubleshooting/network （Network, proxy, and remote connections）
- https://forum.cursor.com/t/what-are-the-urls-needed-to-whitelist-them-in-the-corporate-firewall/15462

## 官方推荐的通配模式（原文）

> Rather than maintaining IP address lists (which can change), configure your firewall to
> allow traffic to these domain patterns:
>
> - `*.cursor.sh`
> - `*.cursor-cdn.com`
> - `*.cursorapi.com`
> - `*.cursorvm.com`
> - `*.*.cursorvm.com`

## 官方精确主机清单（原文，用于禁止通配的防火墙）

> However, if your firewall mandates granular subdomain allowlists without wildcards,
> use the following list:

| 主机                                                                                                                   | 官方用途                                                                          |
| ---------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| `api2.cursor.sh`                                                                                                       | Used for most API requests                                                        |
| `api5.cursor.sh`                                                                                                       | Used for Cursor's agent requests / network access layer (NAL) requests            |
| `api3.cursor.sh`                                                                                                       | Used for Cursor Tab requests (HTTP/2 only)                                        |
| `repo42.cursor.sh`                                                                                                     | Used for codebase indexing (HTTP/2 only)                                          |
| `api4.cursor.sh`、`us-asia.gcpp.cursor.sh`、`us-eu.gcpp.cursor.sh`、`us-only.gcpp.cursor.sh`                           | Used for Cursor Tab requests depending on your location (HTTP/2 only)             |
| `adminportal42.cursor.sh`                                                                                              | Used to configure SSO and domain verification                                     |
| `marketplace.cursorapi.com`、`cursor-cdn.com`、`downloads.cursor.com`、`anysphere-binaries.s3.us-east-1.amazonaws.com` | Used for client updates and downloading extensions from the extension marketplace |

`api5.cursor.sh` 的子域（官方逐条列出）：

```
agent.api5.cursor.sh
agentn.api5.cursor.sh
agent.us.api5.cursor.sh
agentn.us.api5.cursor.sh
agent.global.api5.cursor.sh
agentn.global.api5.cursor.sh
```

认证主机（官方逐条列出）：

| 主机                            | 官方用途                  |
| ------------------------------- | ------------------------- |
| `authenticate.cursor.sh`        | Authorization endpoint    |
| `authenticator.cursor.sh`       | Auth UI and login webview |
| `prod.authentication.cursor.sh` | Production token issuer   |
| `authentication.cursor.sh`      | JWT issuer (backend)      |

## 关键结论

1. **三个认证主机名都真实存在**，不是拼写错误：`authenticate.cursor.sh`（授权端点）、
   `authenticator.cursor.sh`（登录 UI）、`authentication.cursor.sh`（JWT 签发后端）。
   官方另列 `prod.authentication.cursor.sh`，只有 `DOMAIN-SUFFIX,authentication.cursor.sh`
   才能覆盖它。
2. **`gcpp.cursor.sh` 使用后缀匹配是必要的**：官方主机是 `us-asia.`、`us-eu.`、`us-only.`
   三个前缀形式，精确匹配会全部漏掉。
3. **`api5.cursor.sh` 使用后缀匹配是必要的**：官方列出六个 `agent*` 子域。
4. **`cursorvm.com` 使用后缀匹配与官方 `*.cursorvm.com` / `*.*.cursorvm.com` 一致。**
5. `adminportal42.cursor.sh` 是官方给出的唯一 adminportal 主机。脚本的
   `^adminportal[0-9]+\.cursor\.sh$` 是本项目的前向兼容策略，**无官方通配依据**，
   `docs/routing-scope.md` 已如实记录。
6. `repo42.cursor.sh` 官方用途为 codebase indexing，脚本默认关闭该路由，与「不代理多余流量」一致。
7. 市场、CDN、更新下载四项官方明确归类为 client updates / extension marketplace，
   脚本正确排除。

## 逐条判定

| 脚本条目                          | 规则类型             | 官方出处                                   | 判定                                                                           |
| --------------------------------- | -------------------- | ------------------------------------------ | ------------------------------------------------------------------------------ |
| `api2.cursor.sh`                  | DOMAIN-SUFFIX        | 官方精确主机                               | 可收窄为 DOMAIN。官方未列该主机的子域                                          |
| `api5.cursor.sh`                  | DOMAIN-SUFFIX        | 官方列出六个 `agent*` 子域                 | 保留后缀。必需                                                                 |
| `gcpp.cursor.sh`                  | DOMAIN-SUFFIX        | 官方列出三个区域前缀                       | 保留后缀。必需                                                                 |
| `authenticate.cursor.sh`          | DOMAIN-SUFFIX        | 官方精确主机                               | 可收窄为 DOMAIN                                                                |
| `authentication.cursor.sh`        | DOMAIN-SUFFIX        | 官方另列 `prod.authentication.cursor.sh`   | 保留后缀。必需                                                                 |
| `cursorvm.com`                    | DOMAIN-SUFFIX        | 官方 `*.cursorvm.com` / `*.*.cursorvm.com` | 保留后缀。与官方一致                                                           |
| `api3.cursor.sh`                  | DOMAIN               | 官方精确主机                               | 保留                                                                           |
| `api4.cursor.sh`                  | DOMAIN               | 官方精确主机                               | 保留                                                                           |
| `authenticator.cursor.sh`         | DOMAIN               | 官方精确主机                               | 保留                                                                           |
| `api.cursor.com`                  | DOMAIN               | 未在官方网络配置文档出现                   | 存疑。CHANGELOG 记录用途为 Cloud Agent / Bugbot AI API，本次检索未复现官方出处 |
| `^adminportal[0-9]+\.cursor\.sh$` | DOMAIN-REGEX         | 官方仅 `adminportal42.cursor.sh`           | 已记录为前向兼容策略。可收窄为 DOMAIN 精确匹配官方主机                         |
| `^repo[0-9]+\.cursor\.sh$`        | 未注入（开关默认关） | 官方 `repo42.cursor.sh` 用于索引           | 与「不代理多余流量」一致                                                       |

## 过度覆盖实测

`DOMAIN-SUFFIX,api2.cursor.sh` 会额外匹配 `*.api2.cursor.sh`；
`DOMAIN-SUFFIX,cursorvm.com` 会匹配 `cursorvm.com` 下全部子域。
`^adminportal[0-9]+\.cursor\.sh$` 匹配 `adminportal0.cursor.sh` 到 `adminportal999.cursor.sh`
的任意编号，官方只有一个 `adminportal42.cursor.sh`。

这些额外覆盖的主机在官方文档中不存在，实际泄漏面为零；属于规则宽于证据，不是当前的功能问题。
