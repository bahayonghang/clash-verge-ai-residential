# 家宽规则优化

用 ResiWatch 历史库证据收窄 `AI-家宽` 路由范围，减少不必要的家宽流量。

## 触发条件

在以下情况使用本 skill：

- 用户要减少家宽流量、关掉长期无命中的路由开关，或判断某条域名规则是否还该走家宽
- 需要从本机 `monitor.sqlite3` 读家宽 host 排名、份额、死规则或越界流量
- 准备改 `clash-verge-ai-residential.local.toml` 的 `routing.*`，或评估公开模板域名清单是否该收窄

不要用本 skill 去改桌面端 UI、解除 `AUTO_DELETE_ENABLED`，或把观测量当成运营商账单。

## 生成器用法

CLI 不读 Clash 脚本。先在仓库根生成输入：

```bash
node skills/residential-rule-tuning/scripts/build-inputs.js <out-dir>
```

产物：

- `rules.json`：`buildInjectedRules()` 的完整模式清单，是模式全集
- `switches.json`：显式 `supported` / `unsupported` 两个清单

生成器对照 `scripts/sync-local-config.js` 的 `routing` 表做完整性检查：受支持键数 + 不支持清单长度必须等于该表开关总数（21）。检查失败时非零退出，不得把缺失开关写成 0。

贴到 issue、PR 或对话记录前，用 `--redact` 重跑 CLI。默认输出不脱敏，因为判读需要真实 host。

## 命令顺序

1. `just monitor-db --help` 确认子命令。
2. 生成 `rules.json` 与 `switches.json`。
3. `just monitor-db --db <monitor.sqlite3> --since <utc> --until <utc> share`
4. `just monitor-db --db <monitor.sqlite3> --since <utc> --until <utc> rank --by host`
5. `just monitor-db --db <monitor.sqlite3> --since <utc> --until <utc> audit --rules rules.json --map switches.json`
6. 需要维护时用 `maint status` / `retention` / `backup`；`restore` / `vacuum` / `purge` 必须 `--offline-confirmed`，且 CLI 不验证 ResiWatch 是否已退出。

`rank` 是 Top N 诊断视图，不返回份额，也不与 `share` 互相承诺一致性。需要守恒的结论只信 `audit`。

## 四类结果判读

- **死规则 (`dead`)**：窗口内零命中的期望模式。字节为 `0` 且带 `zeroFlow`。可考虑关闭对应开关或移出公开清单，但先排除采集缺口。
- **越界 (`uncovered`)**：走了家宽但不匹配任何受支持域名模式的 host。按进程规则 / IP 规则 / 其余三类展开。不是「观测规则集合」，数据库不保存 `rulePayload`。
- **开关聚合**：`mapped` 唯一归属；`shared` 跨开关重复只计一次；`unmapped` 命中但不在受支持集合（例如始终启用的核心域）；`unsupportedSwitch` 只给状态不给数值。
- **未知 vs 已知空**：无覆盖或能力不支持 → `null` 加状态，退出码 6 表示能力不支持；零流量开关/死规则 → `0` 加 `zeroFlow`。截断时守恒字段为 `null`。

判读细节见 [reference.md](reference.md)。

## 改动落点

- **个人调优**：改本地 `clash-verge-ai-residential.local.toml` 的 `routing.*`，然后 `just render-local`。不要手改 `*.local.js`。
- **公开模板域名清单**：需要官方出处或脱敏 Connections 证据，加 negative test，并通过 `just ci`。

## 禁止项

- 不改 `*.local.js` 生成物
- 不把真实凭据写进公开模板（`HOME_PROXY_TEMPLATE` 保持 `"xxx"` / `""`）
- 不新增宽泛 provider 后缀、marketplace/CDN、遥测域名
- 不把未知写成 0，不把 `rank` 当全量审计
- 不提交 `*.local.toml` / `*.local.js`
