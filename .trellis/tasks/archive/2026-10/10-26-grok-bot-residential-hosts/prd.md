# 把本机 Grok Bot 实测域名补进家宽路由

## Goal

本机 Grok Bot 和 `grok.exe` 已经在走的业务主机继续走家宽。另外把官方登录说明里、当前模板还没覆盖的四台精确登录主机也送进家宽，且不带上它们的父域。

## Confirmed facts

- 2026-10-26 活动连接：`Grok Bot.exe` 为 `api2.cursor.sh`（早些时候还有 `api3.cursor.sh`）；`grok.exe` 为 `grok.com` 与 `cli-chat-proxy.grok.com`。远端都在 FlClash TUN fake-IP 上。运行配置已把这些主机指到 `AI-家宽`，该组只有住宅 SOCKS5。
- 这些实测主机已经在公开模板里。不新增重复规则。
- 用户确认额外加入四台精确主机：`accounts.x.ai`、`accounts.spacex.ai`、`auth.grokusercontent.com`、`auth.grokipedia.com`。出处是 docs.x.ai 的登录域说明（2026-10-30 登录页迁移的最小主机名单）。本次抓包没有看到这四台。
- 本机 TLS 握手成功，证书是父域通配符。`accounts.x.ai` 与 `slack.hooks.x.ai` 共用 `*.x.ai` 证书。因此只注入 `DOMAIN` 精确规则，不注入 `*.x.ai`、`*.spacex.ai`、`*.grokusercontent.com`、`*.grokipedia.com`。
- CDN、遥测、共享存储、apex `x.ai` 维持不走家宽。不新开路由开关。

证据：`research/grok-bot-live-hosts.md`。

## Requirements

1. `routing.grok_core` 开启时，注入上述四台主机的 `DOMAIN` 规则和对应的精确住宅 DNS 策略；关闭时连规则和 DNS 键一起清掉。
2. `routing.grok_web_assets` 不控制这四台主机。
3. 父域、`www` 兄弟主机和 `slack.hooks.x.ai` 不走家宽。
4. 已覆盖的实测主机不重复添加。不改 `*.local.toml` / `*.local.js`，不改正在运行的 FlClash 配置。

## Acceptance Criteria

- [ ] 默认规则包含四条 `DOMAIN,<host>,AI-家宽`，`nameserver-policy` 使用裸主机名，不出现 `+.<父域>`。
- [ ] `routing.grok_core = false` 时四台主机不再命中；`routing.grok_web_assets = false` 时仍然命中。
- [ ] 负向主机不命中：`x.ai`、`www.x.ai`、`slack.hooks.x.ai`、`spacex.ai`、`www.spacex.ai`、`grokusercontent.com`、`www.grokusercontent.com`、`grokipedia.com`、`www.grokipedia.com`。
- [ ] 默认 `AI-家宽` 规则数从 55 增至 59，且无重复。
- [ ] 中文与英文配置/范围文档、README、更新日志写明这四台精确主机以及明确排除的父域。
- [ ] `npm test` 通过。根目录路由变更还要跑 `just ci`。

## Out of scope

- 重写订阅、开启全局、手改本机 FlClash 配置或本地 TOML。
- 把 `*.cursor.sh`、CDN、下载域、Mixpanel、Statsig 或共享存储送进家宽。
- 为这四台主机新增独立开关。
