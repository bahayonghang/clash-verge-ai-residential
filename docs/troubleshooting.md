# Troubleshooting

## No usable upstream was found

Symptoms include an exception mentioning `dialer-proxy`, Profile candidates, or `MATCH/FINAL`.

Check:

- the actual top-level group name in the generated Profile;
- `PROFILE_UPSTREAM_OVERRIDES` spelling, spaces, and emoji;
- whether the selected upstream name contains `#` or `&`; these delimit Mihomo's DoH URL fragment and are rejected rather than encoded;
- whether the Profile has a final `MATCH` or `FINAL` rule;
- whether the candidate is `DIRECT`, `REJECT`, `AI-家宽`, or `家宽-SOCKS5`.

## Reserved-name collision

The names `AI-家宽` and `家宽-SOCKS5` are managed by the script. Rename any unrelated Profile proxy or group that already uses either name.

## Recursive proxy-group error

The selected upstream graph eventually references itself, `AI-家宽`, or `家宽-SOCKS5`. Remove the reference from the subscription override or select a clean top-level airport group. The script also adds exclusion filters to `include-all` groups, but it cannot safely repair every arbitrary group graph.

## Placeholder credential error

The public template keeps `server`, `username`, and `password` as `xxx`. Either:

- edit the ignored `clash-verge-ai-residential.local.toml` and regenerate with `just render-local` or `node scripts/sync-local-config.js`; or
- predefine an existing `家宽-SOCKS5` node in the Profile so the script can reuse its endpoint and credentials.

For no-auth SOCKS5, both credential fields must be empty strings. Never hand-edit the generated `.local.js`; change the TOML and render again.

## AI service works but assets do not

This can be expected under AI-only routing. Marketplace, update, download, media, analytics, and shared dependencies use the original Profile route. Inspect the failed host before widening scope. Prefer one exact domain over a broad provider suffix.

## Cursor Marketplace or YouTube hits AI-家宽

从 v5.6 起，Cursor 核心路由默认启用。从 v5.9.0 起，仓库索引主机 `repo[0-9]+.cursor.sh` 改由 `routing.cursor_repository_indexing` 控制，默认是 `false`，回落原 Profile / 机场上游。缺失的本地 TOML 字段会按 `false` 补全；显式设为 `true` 可恢复 v5.8.1 的 repo 家宽路由。即使启用 Cursor 核心路由，Marketplace 和 YouTube 仍不在分流范围内；如需让 Cursor 核心流量也使用机场上游，请在本地 TOML 中设置 `routing.cursor_core = false`。

若 `repo42.cursor.sh` 仍命中 `AI-家宽`，先检查是否把 `routing.cursor_repository_indexing` 设为 `true`，以及订阅或 Merge 层是否残留用户自有的 `DOMAIN,repo42.cursor.sh,AI-家宽`。Privacy Mode 不会停止索引上传。`disableHttp2` 或服务端强制 HTTP/1.1 时，RepositoryService 可能改走共享的 `api2.cursor.sh`；该主机仍由 `cursor_core` 控制，Clash 域名规则无法在保留多数 Cursor API 的同时隔离这条回退路径。默认关闭索引家宽不能宣称已排除全部仓库上传。

Clash Verge 脚本控制台只显示 `Script execution failed` 时，查看 `%APPDATA%\io.github.clash-verge-rev.clash-verge-rev\logs\latest.log`。`Script execution error: expected value at line 1 column 1` 表示 `main` 抛错后返回值为空。最常见原因是把公开模板 `clash-verge-ai-residential.js`（`HOME_PROXY_TEMPLATE` 为 `xxx`）粘进 Global Extend Script，而当前 Profile 没有预置 `家宽-SOCKS5` 节点。应粘贴 `just render-local` 生成的 `clash-verge-ai-residential.local.js`。

意外命中的常见原因包括：

- stale rules remain in a subscription, another script, or Global Extend Config (Merge);
- a broad user rule such as `DOMAIN-SUFFIX,cursor.com,AI-家宽` exists outside this script;
- process-wide fallback was manually enabled;
- the running Profile was not refreshed after editing the global script.

v5.5 replaces only rules that the current version can generate. It deliberately preserves unknown rules and no longer migrates pre-v5.4 output. If v5.4 output was manually persisted, the following retired Cursor rules are also user-owned and require manual removal:

```text
DOMAIN,repo42.cursor.sh,AI-家宽
DOMAIN-REGEX,^[a-z0-9-]+\.api5\.cursor\.sh$,AI-家宽
DOMAIN-REGEX,^(?:us-asia|us-eu|us-only)\.gcpp\.cursor\.sh$,AI-家宽
```

Also search for broader old or custom entries such as:

```text
DOMAIN-SUFFIX,cursor.com,AI-家宽
DOMAIN,www.youtube.com,AI-家宽
DOMAIN,marketplace.cursorapi.com,AI-家宽
```

Identify which enhancement layer supplied each match, remove stale entries from that source, then refresh the Profile. Do not add the retired strings to the current script merely to clean them up.

## DNS leak test does not show the residential location

This is expected for generic test domains. The script sends only AI-domain DNS through `AI-家宽`; ordinary overseas DNS uses the current airport upstream. Validate an AI hostname in Mihomo DNS logs or connection metadata instead.

## The first connection to a new non-AI domain is slower

Strict DNS rebuilding sends real non-AI overseas lookups through DoH bound to the current Profile upstream. When a GEOIP fallback needs a real lookup, the first query for a new domain can add roughly one airport round trip; cache hits do not pay the same setup cost. This trade-off is retained to keep resolver routing consistent and resistant to pollution. See [DNS and Leak Model](dns-and-leak-model.md).

## Chat/voice or realtime feature fails

The default AI-only policy does not capture generic STUN/TURN or all realtime UDP ports. Confirm the exact product host and the UDP capability of both the airport path and residential SOCKS5 service before enabling shared realtime switches.

## 全新离线安装时配置验证失败（geosite.dat）

重建后的 DNS 策略包含 `geosite:cn` 和 `geosite:private`，两者依赖 `geosite.dat`。Mihomo 首次使用时会下载该文件；设备离线且没有已有副本时，配置解析会失败，Clash Verge Rev 会报告验证错误。若发生在首次启动，应用会回退到最小默认配置。请让设备联网一次，使 Mihomo 能够获取地理数据库（大多数订阅配置也会触发相同下载）；也可以将有效的 `geosite.dat` 放入 Mihomo 工作目录，然后刷新 Profile。

## 脚本中的 TUN DNS 劫持和 IPv6 设置未生效

当前版本的 Clash Verge Rev 会在全局脚本运行后，将控制平面字段（`tun`、`ipv6`、模式、端口）恢复为应用设置值。因此，脚本补全的 TUN DNS 劫持和 `ipv6: false` 在这些宿主上不会生效；相关逻辑仅用于兼容旧版宿主。请改为在 Clash Verge Rev 设置页面配置 IPv6 开关和 TUN DNS 劫持。脚本重建的 DNS 服务器、`nameserver-policy` 和 fake-ip 字段不受影响；但启用 Clash Verge Rev 的 DNS 覆盖后，`dns.ipv6` 也会从应用设置恢复。

## 警告提示已从上游组移除引用

从解析后的 `dialer-proxy` 可达的上游图不得包含 `AI-家宽` 或 `家宽-SOCKS5`，否则链路会递归。脚本发现此类引用时会将其移除，并在警告中记录组名和被移除的条目。请使用目标为 `AI-家宽` 的规则路由 AI 流量，不要将该组嵌套在上游选择器中。若需要自定义 AI 选择器，请确保它不在 `家宽-SOCKS5` 的上游图中。

## 私有 CIDR 规则覆盖自定义内网路由

脚本会在用户规则之前插入回环地址和 RFC1918 网段的直连规则，因为这些规则必须位于所有进程回退规则之前。如果 Profile 有意通过企业代理组转发 `10.0.0.0/8` 等私有网段，注入的 `DIRECT` 规则会优先匹配。这是 fail-closed 设计取舍；请调整对受影响网段的路由预期，或为该 Profile 禁用脚本。
