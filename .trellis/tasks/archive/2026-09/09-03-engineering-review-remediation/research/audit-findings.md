# 2026-09-03 全库工程级 Review 证据

审计层级：deep。范围：第一方源码（根扩展脚本、`scripts/`、`tests/`、`residential-monitor/src` 与 `src-tauri/src`、`skills/`）。排除：`node_modules`、`target/`、`dist/`、`ref/neko-master`、`bench-data/`、gitignored `*.local.toml` / `*.local.js`。无既有 `docs/audits/` 基线。

产品代码本轮未改。已归档任务 `09-03-test-coverage-gaps` 覆盖的缺测不重复立项。

## 架构草图

```
clash-verge-ai-residential.js  → Clash Verge main(config)
scripts/sync-local-config.js   → 单向渲染 *.local.js

ResiWatch
  WebView (React)  --IPC-->  lib.rs generate_handler
                               Mutex<AppFacade>          单锁
                               C1 StorageCoordinator     单 writer SQLite
                               C2 hub/collector/settings
                               C3 ReportService 独立 reader
                               C4 AlertEngine
                               C5 delete/vacuum/about
                               Windows Credential Manager
```

信任边界：本机 loopback 控制器、本机 SQLite、本机凭据库、Tauri WebView。无云、无遥测、无自动更新。

## 命令与工具

| 项 | 结果 |
| --- | --- |
| `git check-ignore` 本地 TOML/JS | 已忽略，未跟踪 |
| `TODO`/`FIXME` 生产源 | 仅公开模板 `HOME_PROXY_TEMPLATE` 四条占位注释 |
| SQL 字符串插值用户值 | `{filters}`/`{order_by}`/`{identities}` 为白名单或 `?` 占位 |
| C2 `use rusqlite` | 无 |
| `dangerouslySetInnerHTML` 产品代码 | 无 |
| `npm audit` / `cargo audit` | 未跑（需网络）`missing evidence` |
| 浏览器/安装态 | 未跑 `missing evidence` |

## 发现索引

见父任务 `prd.md` 任务表。每条 high 均有 `file:line` 已在本会话对照源码。
