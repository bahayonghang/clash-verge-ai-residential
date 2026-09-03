# 家宽规则优化：判读细节

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
