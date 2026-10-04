# 脚本停止写入 Verge 设置页管理的字段

## Goal

把 `clash-verge-ai-residential.js` 粘贴到 Clash Verge Rev v2.5.5+ 后，不再弹出
「Extensions wrote tun.dns-hijack, which Settings manages, so those values were
discarded」提示。脚本改为只读取这些字段并在不安全时输出 `warn`，由用户在设置页配置。

## 原因

Clash Verge Rev v2.5.5 起，扩展执行前后比较设置页管理的字段；值被改动就弹出提示并还原。
脚本 `hardenTun` 向 `tun.dns-hijack` 追加 `any:53`、`tcp://any:53`，因此触发提示。
同一机制也覆盖 `tun.strict-route` 与顶层 `ipv6`。详见
`research/verge-discarded-keys-notice.md`。

## Requirements

- R1 脚本不再写 `tun` 下任何键，也不再写顶层 `ipv6`。
- R2 `tun.enable === true` 且 `tun.dns-hijack` 缺少 `any:53` 或 `tcp://any:53` 时，
  输出一条 `warn`，说明缺少的条目并指向 Verge 的 TUN 设置页。条目已齐全时不输出。
  `harden_existing_tun_dns_hijack = false` 时跳过此检查。
- R3 `harden_existing_tun_dns_hijack` 与 `enable_tun_strict_route` 均为 true、TUN 已开启、`tun.strict-route !== true` 时，
  输出一条 `warn` 指向 TUN 设置页。
- R4 顶层 `ipv6 === true` 时输出一条 `warn`，指向设置页 IPv6 开关。其他值不输出。
- R5 删除每次运行都输出的「新版会还原 tun/ipv6」`info` 提示；由 R2–R4 的条件告警取代。
- R6 保留 `HARDEN_EXISTING_TUN_DNS_HIJACK` / `ENABLE_TUN_STRICT_ROUTE` 常量与
  `runtime.harden_existing_tun_dns_hijack` / `runtime.enable_tun_strict_route` TOML 键，
  现有 `*.local.toml` 渲染不报错。更新 `*.local.toml.example` 注释说明新语义。
- R7 文档与 spec 更新宿主行为：v2.5.5+ 的丢弃提示；DNS 覆盖开启时 `dns_config.yaml`
  非空字段全部以设置页为准（不止 `dns.ipv6`）。中文 `docs/`、英文 `docs/en/`、
  `.trellis/spec/frontend/hook-guidelines.md`、`CHANGELOG.md` `[未发布]`。
- R8 `main` 保持幂等（宿主可能对同一配置执行两次）。

## Out of scope

- 不改 DNS 重建逻辑、路由规则、代理组。
- 不识别宿主版本，不读 `dns_config.yaml`。
- 不改 ResiWatch。

## Acceptance Criteria

- [x] 输入 `tun: { enable: true, "dns-hijack": ["udp://any:53"] }` 时，输出的
      `tun` 与输入深度相等，且产生一条提及 `any:53`、`tcp://any:53` 的 `warn`。
- [x] 输入 `dns-hijack` 已含 `any:53` 与 `tcp://any:53` 时，不产生 TUN 相关 `warn`。
- [x] 输入 `ipv6: true` 时输出 `ipv6` 仍为 `true`，且产生一条 IPv6 `warn`；
      输入无 `ipv6` 或为 `false` 时输出不新增/不改变 `ipv6`，且无 IPv6 `warn`。
- [x] 输入无 `tun` 或 `tun.enable !== true` 时不产生 TUN `warn`，输出不新增 `tun`。
- [x] 模拟 Verge 丢弃检测：对任意测试输入，脚本输出中 `tun` 与顶层 `ipv6` 与输入相同。
- [x] 脚本执行两次结果一致（现有幂等测试继续通过）。
- [x] 原有 `hardenTun` 补全断言改为新行为断言；`npm run ci` 通过。
- [x] `npm --prefix docs run build` 通过。
- [ ] 真实 Clash Verge Rev v2.5.5+ 粘贴脚本后不再出现 `tun.dns-hijack` 丢弃提示
      （由用户手动确认）。

2026-10-03：`npm run ci` 227/227 通过，模板安全检查通过；`npm --prefix docs run build` 通过。真实宿主确认待用户执行。
