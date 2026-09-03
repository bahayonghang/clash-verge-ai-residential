# 依赖扫描与分批升级

## Goal

摸清仓库全部产品依赖的过期、安全、废弃 API、可安全升级与 Breaking Change 边界；按风险从低到高分批升级。每一批在 `just ci` 为 0 之后才进入下一批。用户先看到计划，再批准实施。

## Background

2026-09-03 扫描。根扩展脚本零第三方依赖。文档站只有 VitePress 2 alpha。ResiWatch 前端 npm + 后端 Cargo 是升级面。GitHub Actions 仍钉在 v4。`ref/neko-master/` 是参考树。证据见 `research/inventory.md`。

## Requirements

- R1 升级计划写入本任务 `prd.md` / `design.md` / `implement.md` 与 `research/inventory.md`。批准前不改产品 lockfile、specifier、工作流或源码。
- R2 范围内、无 Breaking 的 lock 更新先做：monitor `typescript-eslint` 8.67.0→8.69.0、`@types/react-dom` 19.2.4→19.2.5；`cargo update` 收 hyper / http-body-util / tauri-plugin-dialog / tauri-plugin-notification 等兼容版本。
- R3 Breaking 按风险分批，一批一次、互不混 lock。顺序见 Key decisions。任一批 `just ci` 非 0 则定位修复或回退该批，不得带着红门进入下一批。
- R4 根 `package.json` 保持无第三方依赖，`engines.node` 保持 `>=18`。CI 根作业继续覆盖 Node 18/20/22。
- R5 不引入新运行时依赖。不升级 `ref/neko-master/`。不把 VitePress 从 `next` 降到 `latest` 1.6.4。
- R6 被 peer 阻断的主版本本轮不做：ESLint 10（`eslint-plugin-react@7.37.5` 不含 eslint 10）、TypeScript 7（`typescript-eslint` 要求 `<6.1.0`）、`@vitejs/plugin-react` 6。
- R7 每一批可独立回退。不改公开模板凭据、不改路由域名、不改核算口径。

## Acceptance Criteria

- [ ] AC1 `research/inventory.md` 列出根 / docs / monitor npm / Cargo / Actions 的当前、Wanted/latest、安全结论与阻断原因。
- [ ] AC2 批准后的实施按批次进行；每批结束后 `just ci` 退出码 0。
- [ ] AC3 根脚本与 `npm run ci` 在 Node 18 契约下仍通过（由 `just ci` 的 `npm run ci` 覆盖本机 Node；矩阵仍由 GitHub `test` 作业覆盖）。
- [ ] AC4 本轮结束时：范围内 lock 已更新；已做批次的 Breaking 已落地或已在任务记录中标明回退；R6 阻断项仍保持现状并写明原因。
- [ ] AC5 `npm --prefix residential-monitor audit` 仍为 0。Cargo 在 advisory-db 可拉取时重跑 `cargo audit`；拉失败则记录，不把失败当成已发现漏洞。
- [ ] AC6 不提交 `*.local.toml` / `*.local.js`，公开模板凭据仍为占位。

## Out of scope

- 给根扩展脚本增加 npm 依赖或构建器。
- 升级或删除 `ref/neko-master/`。
- VitePress 2 稳定版（尚无 `latest` 2.x）或降回 1.6.4。
- ESLint 10、TypeScript 7、plugin-react 6。
- clap 5 / Tauri 3 / rusqlite 新主版本（扫描日 latest 仍为当前主版本）。
- Rust edition 2021→2024。
- 接入 Dependabot / Renovate。
- 把 `docs-build` 塞进 `just ci`。
- 本机安装 NSIS、30 天库、soak。

## Key decisions

- 产品依赖 = 根脚本工具链 + `docs/` + `residential-monitor` npm/Cargo + `.github/workflows/ci.yml`。参考树排除。
- 无已知需紧急修复的 CVE。范围内更新仍做，为收补丁。
- 批次从低到高：1 lockfile → 2 Actions v7 → 3 lucide-react 1 → 4 Cargo `sha2`/`rand`/`tokio-tungstenite` → 5 react-hooks 7 → 6 TypeScript 6.0.3 → 7 Vitest 4 → 8 Vite 8（保留 plugin-react 5）。
- Vite 8 不捆绑 plugin-react 6。Vitest 4 可在 Vite 7 上先做。
- TypeScript 只升到 6.0.3。Actions 一次跳到 checkout v7 与 setup-node v7，保留 `persist-credentials: false` 与 Node 矩阵。
