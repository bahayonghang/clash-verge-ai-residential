# Design

## 边界

- 生产文件：`clash-verge-ai-residential.js`（`hardenTun`、`main` 第 9 步与末尾 `info`）。
- 本地配置：`clash-verge-ai-residential.local.toml.example` 注释；
  `scripts/sync-local-config.js` 字段表不变。
- 测试：`tests/regression.test.js`。
- 文档：`docs/configuration.md`、`docs/dns-and-leak-model.md`、`docs/troubleshooting.md`
  及 `docs/en/` 对应页；`docs/local-configuration.md` 与 `docs/en/local-configuration.md` 的两行 runtime 说明；`CHANGELOG.md`；`.trellis/spec/frontend/hook-guidelines.md`。

## 行为契约

`hardenTun(config)` 重命名为 `checkHostOwnedFields(config)`（只读），在 `main` 原调用点执行。

```
if HARDEN_EXISTING_TUN_DNS_HIJACK && tun.enable === true:
  missing = ["any:53", "tcp://any:53"] 中不在 tun["dns-hijack"] 的项
  missing 非空 → warn(缺少项 + "请在 Verge 设置 → TUN 设置 → DNS 劫持中添加")
  if ENABLE_TUN_STRICT_ROUTE && tun["strict-route"] !== true → warn(TUN strict-route 设置页)
if config.ipv6 === true → warn(请在 Verge 设置页关闭 IPv6)
```

- 删除 `working.ipv6 = false` 与固定 `info` 提示。
- `warn` 使用脚本已有日志函数，前缀与其他告警一致（`[${AI_GROUP}]`）。
- 读取值来自宿主传入的 `config`。宿主在脚本执行前已写入设置页值，因此检查结果反映设置页状态。
- 不写入任何受管字段，第二次执行输入不变，幂等成立。每次执行最多 3 条告警，不触及 1000 行日志上限。

## 取舍

- 放弃 v2.5.4 及更早宿主上的自动补全。脚本无法识别宿主版本；保留写入会在 v2.5.5+ 持续触发提示且写入无效。用户已选择此方案。
- 保留 TOML 键：`sync-local-config.js` 对未知键抛错，删除键会使现有 `*.local.toml` 渲染失败。
  键名中的 `harden` 语义变为「检查」，在 example 注释中说明。

## 回滚

单提交回滚即可恢复 `hardenTun` 与 `ipv6` 写入；无数据迁移。
