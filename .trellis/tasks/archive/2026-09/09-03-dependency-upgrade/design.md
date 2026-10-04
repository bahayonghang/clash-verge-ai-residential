# 设计：分批依赖升级

## 边界

- 改动只落在各批列出的 lockfile / specifier / 为编译通过所需的最少源码。
- 根 `clash-verge-ai-residential.js`、路由域名、公开模板凭据、SQLite schema、IPC DTO 不因升级而改语义。
- 新 crate / 新 npm 运行时包不进入本任务。

## 兼容

| 约束 | 处理 |
|---|---|
| 根 Node >=18，CI 矩阵 18/20/22 | 不改根 engines；monitor 工具链升到 Vite 8 / ESLint 工具只跑 monitor 作业（Node 22） |
| Vite 8 engines `^20.19.0 \|\| >=22.12.0` | monitor CI 继续 `node-version: "22"`（取当时 22.x）。若 runner 低于 22.12，停在本批并记录 |
| `typescript-eslint` `typescript <6.1.0` | TS 目标 6.0.3 |
| `eslint-plugin-react` 不含 eslint 10 | 留 ESLint 9.39.5 |
| plugin-react 5 已声明 vite 8 | Vite 8 批次不升 plugin-react 6 |
| sha2 0.11 / rand 0.10 / tokio-tungstenite 0.30 的 rust-version 1.85 | 本机 1.98、CI stable 满足 |
| Tauri plugin minor（dialog 2.7.3、notification 2.4.0） | 放进 Batch 1；`NotificationExt` / 对话框 API 编译失败则只回退这两个 crate |

## 数据流 / 契约

升级不改变：

- 前端只经 `hooks/**` `invoke`；Channel 仍在 `lib.rs`。
- C2 不得 `use rusqlite`。
- `Sha256` 十六进制校验和字符串格式保持 lowercase hex，避免档案 checksum 语义变化。
- lucide 图标换包后，现有 `data-*` / `aria-*` / `getByRole` 测试契约保持。

## 批次形状

每一批：改文件 → 该批自己的窄编译（见 implement）→ `just ci` → 绿才开始下一批。失败：只回退该批文件，修到绿或在任务记录标明放弃该批。

GitHub Actions 批在本机无法完整模拟 runner。该批的 `just ci` 证明工作流 YAML 仍被忽略（ci.yml 不被本地门执行）。该批的产品证明是「YAML 仍 checkout + setup-node + 原矩阵」；托管 runner 证明留到推送后的 `CI` 作业。

## 回滚

- Batch 1：还原 `residential-monitor/package-lock.json` 与 `src-tauri/Cargo.lock`（及未改 specifier 的 package.json）。
- 其后每批：还原该批 specifier + lock + 为编译改过的源码。
- 不 `git commit --amend`。实施阶段工作区保持可按批还原。

## 权衡

- Actions v4→v7 一次跳两个主版本，避免在 v5/v6 停留。若 v7 在托管 runner 失败，回退到扫描时的 v4 并记录。
- Cargo 三个 Breaking 放同一批、内部按 sha2 → rand → tungstenite 顺序编译，只跑一次 `just ci`，减少 monitor 全量门次数。
- Vitest 4 与 Vite 8 拆开：Vitest 4 peer 已含 vite 7，失败面更小。
