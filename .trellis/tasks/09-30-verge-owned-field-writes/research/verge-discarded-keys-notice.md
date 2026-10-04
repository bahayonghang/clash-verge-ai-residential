# Clash Verge Rev 丢弃字段提示调研（2026-09-30）

来源：`clash-verge-rev/clash-verge-rev` dev 分支 `src-tauri/src/enhance/mod.rs`、
`src-tauri/src/constants.rs`、`src/locales/en/profiles.json`（经 `gh api` 读取）。

## 提示文本来源

`src/locales/en/profiles.json` 的 `discardedKeys`：
`Extensions wrote {{keys}}, which Settings manages, so those values were discarded`。

## 引入版本

- `7bf8e1d` 2026-09-21 `feat(enhance): warn when extensions write app-owned keys`
- `d374ab4` 2026-09-21 `refactor(enhance): unify discarded-key notice state`
- `6a85d03` / `ff1ce92` / `758300a` 2026-09-22 DNS 覆盖字段优先级调整
- 首个包含上述提交的发行版：v2.5.5（2026-09-22）。当前最新：v2.5.6（2026-09-26）。

## 检测机制

`enhance()` 顺序：app 生成 tun/dns → `AuthoritativeFields::capture` → 全局 Merge/Script
→ Profile Merge/Script → `notify_discarded_keys(overridden(...))` → `enforce`。

`overridden` 按键比较：扩展执行前后值不同，且执行后值不等于快照值，就记为丢弃键。
受管键：

- 顶层 `CONTROL_PLANE_KEYS`：`external-controller*`、`secret`、`mixed-port`、
  `socks-port`、`port`、`redir-port`、`tproxy-port`、`mode`、`allow-lan`、`log-level`、
  `ipv6`、`unified-delay`。
- `tun.enable`，以及设置页已保存的 `constants::tun::GUI_KEYS`：`stack`、`device`、
  `auto-route`、`route-exclude-address`、`auto-redirect`（Linux）、
  `auto-detect-interface`、`dns-hijack`、`strict-route`、`mtu`。
- 启用 DNS 覆盖时：`dns_config.yaml` 中所有非空 `dns.*` 字段，以及非空 `hosts`。

提示按「丢弃键集合」去重：集合与上次相同时不再弹出；集合为空时不弹出。

## 对本脚本的影响

- `hardenTun`（`clash-verge-ai-residential.js:1770`）在 `tun.enable === true` 时向
  `tun.dns-hijack` 追加 `any:53`、`tcp://any:53`。设置页值不含这两项时，值发生变化
  → 触发 `tun.dns-hijack` 提示，且写入被还原。截图中的提示即此路径。
- `ENABLE_TUN_STRICT_ROUTE = true` 时写 `tun.strict-route`，同理会触发提示。默认 false。
- `main` 末尾 `working.ipv6 = false`（`clash-verge-ai-residential.js:1880`）：设置页
  IPv6 为开时触发 `ipv6` 提示。截图未出现，推断用户设置页 IPv6 已关。
- 脚本重建的 `dns` 在 DNS 覆盖关闭时不受管，可存活。DNS 覆盖开启时，
  `dns_config.yaml` 中非空字段全部以设置页为准（不止 `dns.ipv6`）；脚本对这些字段的
  改写会被丢弃并出现 `dns.*` 提示。现有文档与 spec 只写了 `dns.ipv6`，已过时。
- 脚本无法识别宿主版本，也无法读取 `dns_config.yaml`。
