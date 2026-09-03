# 故障排查

## 找不到可用上游

异常信息通常提到 `dialer-proxy`、Profile 候选或 `MATCH/FINAL`。

检查：

- 生成 Profile 里实际的顶层组名；
- `PROFILE_UPSTREAM_OVERRIDES` 的拼写、空格和 emoji；
- 选中的上游名是否含 `#` 或 `&`（它们是 Mihomo DoH URL 片段分隔符，脚本拒绝而不是编码）；
- Profile 是否有最后的 `MATCH` 或 `FINAL` 规则；
- 候选是否为 `DIRECT`、`REJECT`、`AI-家宽` 或 `家宽-SOCKS5`。

## 保留名冲突

`AI-家宽` 和 `家宽-SOCKS5` 由脚本托管。Profile 里无关的代理或组若已占用这两个名字，请改名。

## 代理组递归错误

选中的上游图最终引用了自己、`AI-家宽` 或 `家宽-SOCKS5`。从订阅覆盖里去掉该引用，或改选干净的顶层机场组。脚本会给 `include-all` 组加排除过滤，但不能安全修复任意组图。

## 占位凭据错误

公开模板把 `server`、`username`、`password` 保持为 `xxx`。任选其一：

- 编辑被忽略的 `clash-verge-ai-residential.local.toml`，再用 `just render-local` 或 `node scripts/sync-local-config.js` 重新生成；或
- 在 Profile 里预置已有的 `家宽-SOCKS5` 节点，让脚本复用它的 endpoint 和凭据。

无认证 SOCKS5 必须把两个凭据字段都设成空字符串。不要手改生成的 `.local.js`；改 TOML 再渲染。

## AI 服务可用但资源不可用

这在 AI-only 路由下可以是预期行为。市场、更新、下载、媒体、统计和共享依赖走原 Profile。扩大范围前先看失败的主机。优先加一条精确域名，而不是宽泛的 provider 后缀。

## Cursor Marketplace 或 YouTube 命中 AI-家宽

从 v5.6 起，Cursor 核心路由默认启用。从 v5.9.0 起，仓库索引主机 `repo[0-9]+.cursor.sh` 改由 `routing.cursor_repository_indexing` 控制，默认 `false`，回落原 Profile / 机场上游。缺失的本地 TOML 字段会按 `false` 补全；显式设为 `true` 可恢复 v5.8.1 的 repo 家宽路由。即使启用 Cursor 核心路由，Marketplace 和 YouTube 仍不在分流范围内；若要让 Cursor 核心流量也走机场上游，在本地 TOML 设 `routing.cursor_core = false`。

若 `repo42.cursor.sh` 仍命中 `AI-家宽`，先检查是否把 `routing.cursor_repository_indexing` 设为 `true`，以及订阅或 Merge 层是否残留用户自有的 `DOMAIN,repo42.cursor.sh,AI-家宽`。Privacy Mode 不会停止索引上传。`disableHttp2` 或服务端强制 HTTP/1.1 时，RepositoryService 可能改走共享的 `api2.cursor.sh`；该主机仍由 `cursor_core` 控制，Clash 域名规则无法在保留多数 Cursor API 的同时隔离这条回退路径。默认关闭索引家宽不能宣称已排除全部仓库上传。

Clash Verge 脚本控制台只显示 `Script execution failed` 时，查看 `%APPDATA%\io.github.clash-verge-rev.clash-verge-rev\logs\latest.log`。`Script execution error: expected value at line 1 column 1` 表示 `main` 抛错后返回值为空。最常见原因是把公开模板 `clash-verge-ai-residential.js`（`HOME_PROXY_TEMPLATE` 为 `xxx`）粘进 Global Extend Script，而当前 Profile 没有预置 `家宽-SOCKS5` 节点。应粘贴 `just render-local` 生成的 `clash-verge-ai-residential.local.js`。

意外命中的常见原因：

- 订阅、另一段脚本或 Global Extend Config (Merge) 里还有旧规则；
- 脚本之外存在宽规则，例如 `DOMAIN-SUFFIX,cursor.com,AI-家宽`；
- 手动打开了进程级兜底；
- 改完全局脚本后没有刷新正在运行的 Profile。

v5.5 只替换当前版本能生成的规则。它故意保留未知规则，也不再迁移 pre-v5.4 输出。若 v5.4 输出曾被手工持久化，下列已退役 Cursor 规则也视为用户所有，需要手工删除：

```text
DOMAIN,repo42.cursor.sh,AI-家宽
DOMAIN-REGEX,^[a-z0-9-]+\.api5\.cursor\.sh$,AI-家宽
DOMAIN-REGEX,^(?:us-asia|us-eu|us-only)\.gcpp\.cursor\.sh$,AI-家宽
```

同时搜索更宽的旧规则或自定义规则，例如：

```text
DOMAIN-SUFFIX,cursor.com,AI-家宽
DOMAIN,www.youtube.com,AI-家宽
DOMAIN,marketplace.cursorapi.com,AI-家宽
```

确认是哪一层增强写入了匹配项，从该源删掉残留，再刷新 Profile。不要把退役字符串加回当前脚本仅仅为了清理。

## DNS leak test 没有显示住宅地区

对通用测试域名这是预期。脚本只把 AI 域名的 DNS 送进 `AI-家宽`；普通海外 DNS 走当前机场上游。请在 Mihomo DNS 日志或连接元数据里验证一个 AI 主机名。

## 第一次访问新的非 AI 域名更慢

严格 DNS 重建会把真实的非 AI 海外查找经绑定到当前 Profile 上游的 DoH 发出。GEOIP 回退需要为新域名做第一次真实查找时，大约多一次机场往返；缓存命中不再付同样成本。该取舍是为了解析一致和抗污染。见 [DNS 与泄漏模型](dns-and-leak-model.md)。

## 聊天/语音或实时功能失败

默认 AI-only 策略不捕获通用 STUN/TURN，也不捕获全部实时 UDP 端口。打开共享实时开关之前，先确认产品的确切主机，以及机场路径和住宅 SOCKS5 的 UDP 能力。

## 全新离线安装时配置验证失败（geosite.dat）

重建后的 DNS 策略包含 `geosite:cn` 和 `geosite:private`，两者依赖 `geosite.dat`。Mihomo 首次使用时会下载该文件；设备离线且没有已有副本时，配置解析会失败，Clash Verge Rev 会报告验证错误。若发生在首次启动，应用会回退到最小默认配置。请让设备联网一次，使 Mihomo 能够获取地理数据库（大多数订阅配置也会触发相同下载）；也可以将有效的 `geosite.dat` 放入 Mihomo 工作目录，然后刷新 Profile。

## 脚本中的 TUN DNS 劫持和 IPv6 设置未生效

当前版本的 Clash Verge Rev 会在全局脚本运行后，将控制平面字段（`tun`、`ipv6`、模式、端口）恢复为应用设置值。因此，脚本补全的 TUN DNS 劫持和 `ipv6: false` 在这些宿主上不会生效；相关逻辑仅用于兼容旧版宿主。请改为在 Clash Verge Rev 设置页面配置 IPv6 开关和 TUN DNS 劫持。脚本重建的 DNS 服务器、`nameserver-policy` 和 fake-ip 字段不受影响；但启用 Clash Verge Rev 的 DNS 覆盖后，`dns.ipv6` 也会从应用设置恢复。

## 警告提示已从上游组移除引用

从解析后的 `dialer-proxy` 可达的上游图不得包含 `AI-家宽` 或 `家宽-SOCKS5`，否则链路会递归。脚本发现此类引用时会将其移除，并在警告中记录组名和被移除的条目。请使用目标为 `AI-家宽` 的规则路由 AI 流量，不要将该组嵌套在上游选择器中。若需要自定义 AI 选择器，请确保它不在 `家宽-SOCKS5` 的上游图中。

## 私有 CIDR 规则覆盖自定义内网路由

脚本会在用户规则之前插入回环地址和 RFC1918 网段的直连规则，因为这些规则必须位于所有进程回退规则之前。如果 Profile 有意通过企业代理组转发 `10.0.0.0/8` 等私有网段，注入的 `DIRECT` 规则会优先匹配。这是 fail-closed 设计取舍；请调整对受影响网段的路由预期，或为该 Profile 禁用脚本。
