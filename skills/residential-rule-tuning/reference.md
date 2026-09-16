# 家宽规则优化：判读细节

源库：`skills/residential-rule-tuning/`。项目级平台副本可能落后于源库。适用工具见 SKILL.md。

## CLI 边界

`monitor-db` 只接触 residential-monitor 与其数据库。它不读、不解析、不改写 `clash-verge-ai-residential.js`、`*.local.toml` 或 `*.local.js`。

库路径：`--db` → `RESIDENTIAL_MONITOR_DATA_DIR` → `%LOCALAPPDATA%\ResiWatch\data\monitor.sqlite3`。不触发数据目录迁移，库缺失时不创建空库（退出码 3）。

## 模式匹配

| 前缀 | 语义 |
| --- | --- |
| `DOMAIN` | 小写全等 |
| `DOMAIN-SUFFIX` | host 等于模式，或以 `.` + 模式结尾（标签边界） |
| `DOMAIN-REGEX` | `regex::Regex::is_match`，部分匹配，锚点由模式自己的 `^` `$` 决定 |

编译失败或未知类型进入 `unsupportedPattern`，不判 dead，也不把 host 判为该模式的 covered。

`notclaude.ai` 不命中 `DOMAIN-SUFFIX,claude.ai`。`us-central1-aiplatform.googleapis.com` 命中 Vertex 区域正则；`aiplatform.googleapis.com` 不命中该正则。

匹配优先级 exact > 最长 suffix > regex > 输入顺序。这是本工具的字节归属规则，**不模拟 Mihomo 首个规则命中**。

## 核心开关与统计归属

Unreleased 增加四个默认 `true` 的域名开关：`anthropic_core` 对应 Claude/Anthropic 产品、API、MCP 与会话域；`gemini_api_core` 对应 `generativelanguage.googleapis.com`；`antigravity_core` 对应四个 Antigravity/Code Assist 核心 exact 主机；`extra` 当前对应 `anyrouter.top`。它们进入 supported 后，总数为 25 个 routing 开关，其中 13 supported、12 unsupported。默认开关未收窄，不能据此声称节省家宽流量。

`anthropic_core=false` 会停止 Anthropic CIDR 与专属进程兜底；OpenAI、Antigravity、Cursor 的进程兜底也服从各自 core。对应 IP/进程开关的统计能力仍是原有 unsupported 状态，不因域名归属增加而获得数值。认证、辅助、静态资源及全局实时/DNS开关仍独立。

生成器的 `rules.json` 来自公开模板，`switches.json` 是域名归属表；二者不反映用户当前启用状态。数据库没有 rulePayload，host 的匹配字节不能证明由某个开关造成。关闭某项只撤销脚本托管捕获，原 Profile 和用户自定义家宽规则仍可能匹配；若要判断真实收益，应检查采集覆盖、实际配置和相近工作负载，不把 rank 当全量审计。

## 守恒

未截断且能力可用时：

```
Σcovered(=Σmapped + shared + unmapped) + ΣunsupportedPattern + Σuncovered = 窗口内家宽总字节
```

`dead` 恒为 0。`uncovered` 是 host 集合，不参与模式集合等式。

## 改动落点

1. 本地 TOML `routing.*` → `just render-local`。生成器完整性检查必须仍然通过。
2. 公开模板域名清单 → 官方出处或脱敏 Connections 证据 + negative test + `just ci`。
3. 新路由域名需要 README / PR 模板要求的出处，默认拒绝宽泛 provider 后缀。

## 禁止项

- 不改 `*.local.js`
- 不把真实凭据写进公开模板
- 不新增宽泛 provider 后缀
- 贴出 CLI 输出前用 `--redact` 重跑
- `restore` / `vacuum` / `purge` 先退出 ResiWatch，再加 `--offline-confirmed`。CLI 不验证该前置条件。`vacuum` 与 `purge` 不可中断。
