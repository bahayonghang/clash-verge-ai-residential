# Implement

1. 测试先行（`tests/regression.test.js`）
   - 改写「已开启 TUN 时只补齐 DNS 劫持…」：断言 `output.tun` 深度等于输入，warn 含缺失项；
     保留 `find-process-mode` 与 PROCESS 规则断言。
   - 新增：dns-hijack 已齐全无 warn；`ipv6: true` 保留且 warn；无 ipv6 时输出无 `ipv6` 键；
     TUN 关闭时无 warn、无新增 `tun`；`enable_tun_strict_route` 语义用常量现值覆盖（若测试无法改常量，只测默认值路径）。
   - 通用断言：输出 `tun`、`ipv6` 与输入相同。
2. 脚本（`clash-verge-ai-residential.js`）
   - `hardenTun` → `checkHostOwnedFields`，按 design 只读告警。
   - 删除 `working.ipv6 = false` 与固定 `info` 提示；更新第 9 步注释。
   - 更新常量 `HARDEN_EXISTING_TUN_DNS_HIJACK` / `ENABLE_TUN_STRICT_ROUTE` 的注释。
3. `clash-verge-ai-residential.local.toml.example`：两键注释改为「检查并告警，不写入」。
4. 文档：`docs/configuration.md`、`docs/dns-and-leak-model.md`、`docs/troubleshooting.md`
   与 `docs/en/` 对应页；说明 v2.5.5+ 丢弃提示、脚本只检查、DNS 覆盖开启时
   `dns_config.yaml` 非空字段全部以设置页为准。
5. `CHANGELOG.md` `[未发布]` → `### 修复` 增一条。
6. `.trellis/spec/frontend/hook-guidelines.md` 控制面条目：补 tun GUI 键、丢弃提示、DNS 覆盖范围；
   规定脚本不得写受管字段。

## 验证

- `npm run ci`
- `npm --prefix docs run build`
- 用户在 Clash Verge Rev v2.5.6 粘贴新脚本，确认无丢弃提示，Script 日志出现预期 warn。

## 回滚点

步骤 2 完成后若测试失败，回退脚本改动，保留测试以复现。
