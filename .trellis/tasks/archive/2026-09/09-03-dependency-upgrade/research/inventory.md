# 依赖扫描（2026-09-03）

扫描日本地工具链：Node v26.7.0、npm 12.0.2、rustc 1.98.0、cargo 1.98.0。CI 根作业 Node 18/20/22；monitor 作业 Node 22 + `dtolnay/rust-toolchain@stable`。

## 清单边界

| 位置 | 清单 | 结论 |
|---|---|---|
| 根 `package.json` | 无 `dependencies` / `devDependencies` | 无可升级 npm 包。`engines.node` 保持 `>=18`。 |
| `docs/package.json` | `vitepress ^2.0.0-alpha.19` | lock `2.0.0-alpha.19` 等于 dist-tag `next`。`latest` 仍是 1.6.4。本轮不升级、不降级。本机无 `docs/node_modules`。 |
| `residential-monitor/package.json` | 前端运行时 + 工具链 | 见 npm 表。 |
| `residential-monitor/src-tauri/Cargo.toml` | Tauri 2 后端 | 见 Cargo 表。 |
| `.github/workflows/ci.yml` | `actions/checkout@v4`、`actions/setup-node@v4` | 最新 checkout `v7.0.1`、setup-node `v7.0.0`。 |
| `ref/neko-master/` | 参考树 | 不在产品构建路径，排除。 |

`just ci` = `monitor-check` + `npm run ci`。文档构建不在该门内。

## npm：residential-monitor

`npm audit`：0。lock 内 `deprecated` 字段：0。

`npm outdated`（Current / Wanted / Latest）：

| 包 | 当前 | 范围内 Wanted | Latest | 判定 |
|---|---|---|---|---|
| `typescript-eslint` | 8.67.0 | 8.69.0 | 8.69.0 | lock 落后，范围内可升 |
| `@types/react-dom` | 19.2.4 | 19.2.5 | 19.2.5 | lock 落后，范围内可升 |
| `lucide-react` | 0.544.0 | 0.544.0 | 1.39.0 | 0 → 1，Breaking |
| `eslint-plugin-react-hooks` | 5.2.0 | 5.2.0 | 7.1.1 | 5 → 7，Breaking；peer 含 eslint 9 与 10 |
| `vitest` | 3.2.7 | 3.2.7 | 4.1.11 | 3 → 4，Breaking；peer 允许 vite 6/7/8 |
| `vite` | 7.3.6 | 7.3.6 | 8.2.2 | 7 → 8，Breaking；engines `^20.19.0 \|\| >=22.12.0` |
| `@vitejs/plugin-react` | 5.2.0 | 5.2.0 | 6.1.1 | 5.2.0 的 peer 已含 vite 8；6.x 额外 peer 均可选 |
| `eslint` / `@eslint/js` | 9.39.5 | 9.39.5 | 10.9.1 / 10.0.1 | 9 → 10，Breaking |
| `typescript` | 5.9.3 | 5.9.3 | 7.0.2 | 5 → 6/7，Breaking |
| 其余直接依赖 | 当前 = Latest | — | — | 本轮不动 specifier |

未出现在 outdated 表中的直接依赖（含 React 19.2.8、Radix、Tailwind 4.3.3、Recharts 3.10.1、`@tauri-apps/api` 2.11.1 / `cli` 2.11.4）视为已在范围内最新。

### peer 阻断

- `typescript-eslint@8.69.0`：`typescript >=4.8.4 <6.1.0`，`eslint ^8.57 \|\| ^9 \|\| ^10`。允许 TS 6.0.x 与 ESLint 10；不允许 TS 7。
- `eslint-plugin-react@7.37.5`（亦为 latest）：`eslint ^3 … ^9.7`，**不含 10**。ESLint 10 本轮不做。
- `@vitejs/plugin-react@5.2.0`：`vite ^4.2 \|\| ^5 \|\| ^6 \|\| ^7 \|\| ^8`。Vite 8 可继续用 plugin-react 5。
- `@vitejs/plugin-react@6.1.1`：`vite ^8`；`oxc-transform-react` / `@rolldown/plugin-babel` / `babel-plugin-react-compiler` 均为 optional。本轮不升 6。
- `eslint-plugin-react-hooks@7.1.1`：eslint `^3 … ^10`。可在 ESLint 9 上升 7。
- `lucide-react@1.39.0`：`react ^16.5 … ^19`。
- TypeScript 6.0.3 存在；7.0.2 engines `node>=16.20`。TS 7 被 ts-eslint peer 挡住。

## Cargo

`cargo audit --file residential-monitor/src-tauri/Cargo.lock`：拉取 `RustSec/advisory-db` 失败（git/IO）。改用 OSV / GitHub Advisory。

`cargo info` 默认 latest 与 lock 对照（直接依赖）：

| crate | lock | crates.io latest | 判定 |
|---|---|---|---|
| chrono | 0.4.45 | 0.4.45 | 已最新 |
| clap | 4.6.6 | 4.6.6 | 已最新（无 clap 5 latest） |
| hex | 0.4.3 | 0.4.3 | 已最新 |
| http | 1.5.0 | 1.5.0 | 已最新 |
| http-body-util | 0.1.4 | 0.1.5 | 范围内 |
| hyper | 1.11.0 | 1.11.1 | 范围内 |
| hyper-util | 0.1.20 | 0.1.20 | 已最新 |
| rand | 0.9.5 | 0.10.2 | Breaking；rust-version 1.85 |
| regex | 1.13.1 | 1.13.1 | 已最新 |
| rusqlite | 0.40.2 | 0.40.2 | 已最新 |
| serde / serde_json | 1.0.229 / 1.0.151 | 同 | 已最新 |
| sha2 | 0.10.9 | 0.11.0 | Breaking；rust-version 1.85 |
| tauri / tauri-build | 2.11.5 / 2.6.3 | 同 | 已最新 |
| tauri-plugin-autostart | 2.5.1 | 2.5.1 | 已最新 |
| tauri-plugin-dialog | 2.7.2 | 2.7.3 | 范围内 |
| tauri-plugin-notification | 2.3.3 | 2.4.0 | 范围内 minor |
| thiserror | 2.0.20 | 2.0.20 | 已最新 |
| tokio | 1.53.1 | 1.53.1 | 已最新 |
| tokio-tungstenite | 0.27.0 | 0.30.0 | Breaking；rust-version 1.85 |
| windows-sys | 0.61.2 | 0.61.2 | 已最新 |
| futures-util (dev) | 0.3.33 | 0.3.34 | 范围内 |
| tempfile (dev) | 3.27.0 | 3.27.0 | 已最新 |

`cargo update --dry-run`：75 个范围内锁定更新。另 7 个 unchanged behind latest：`rand`、`sha2`、`tokio-tungstenite`，以及传递依赖 `generic-array`、`toml` 0.8.2、`toml_datetime`、`toml_edit`。不强制拧传递依赖。

### 本仓库 API 触点

- `sha2::{Digest, Sha256}`：`Sha256::digest` / `Sha256::new`，多文件（c3/c4/c5/storage/workload/dbcli）。
- `rand`：仅 `candidate_schema.rs`，`SmallRng` + `rng.random::<f64>()`。
- `tokio-tungstenite`：仅 `transport.rs` `#[cfg(test)]`，`Message` / `accept_async` / `connect_async`。
- `tauri_plugin_notification::NotificationExt`：`c4/notify.rs`。
- `tauri_plugin_autostart::ManagerExt`：`c2/desktop.rs`。

## 安全

| 源 | 结果 |
|---|---|
| `npm --prefix docs audit` | 0（按 lock，本机无 node_modules） |
| `npm --prefix residential-monitor audit` | 0 |
| OSV `tokio@1.53.1` `rusqlite@0.40.2` `hyper@1.11.0` `rand@0.9.5` `sha2@0.10.9` `tokio-tungstenite@0.27.0` `windows-sys@0.61.2` `vite@7.3.6` `react@19.2.8` `vitepress@2.0.0-alpha.19` | 空对象（无已知漏洞）。`tauri` / `vitest` 查询遇 TLS 失败，未作为「有洞」解释 |
| GHSA `vite@7.3.6` `eslint@9.39.5` `tauri@2.11.5` `rusqlite@0.40.2` `tokio@1.53.1` | length 0 |
| `cargo audit` | 未能刷新 advisory-db |

未发现需紧急安全补丁的直接依赖。范围内 `cargo update` 仍应执行，以收 hyper / plugin 补丁。

## 废弃 API

- 两份 npm lock 无 `deprecated` 包。
- 源码无 `#[deprecated]` / `allow(deprecated)`。
- 根脚本未使用 `url.parse` / `SlowBuffer` / `new Buffer`。
- 前端 Tauri 只用 `@tauri-apps/api/core` 的 `invoke` / `Channel`。
- lucide 具名导出分布在 18 个 tsx；0 → 1 需核对导出名。
- `eslint.config.js` 使用 `reactHooks.configs.recommended.rules`；hooks 7 可能改 flat 导出形状。

## GitHub Actions

- `actions/checkout@v4` → 最新 `v7.0.1`。v7.0.1 相对 v7 为小修补。v4 → v7 跨两个主版本。
- `actions/setup-node@v4` → 最新 `v7.0.0`（changelog 相对 v6：ESM 迁移、cache 输出、去掉 dummy `NODE_AUTH_TOKEN`）。
- 现有 `persist-credentials: false` 与 Node 矩阵必须保留。
- 本机 `just ci` 不执行 GitHub-hosted runner；Actions 批次的完整证明在推送后的 `CI` 工作流。
