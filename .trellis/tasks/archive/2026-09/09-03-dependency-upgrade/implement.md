# Implement：分批依赖升级

## 前置

- 用户批准本规划摘要之后才能 `task.py start`。
- 每批结束后必须 `just ci` 退出码 0。失败则只修/回退该批。
- 不改 `*.local.toml` / `*.local.js` / 公开模板凭据。
- 不改 `ref/neko-master/`。

## 顺序

### Batch 1 — lockfile 范围内（最低风险）

- `npm --prefix residential-monitor update typescript-eslint @types/react-dom`
- `cargo update --manifest-path residential-monitor/src-tauri/Cargo.toml`
- 不改 package.json / Cargo.toml specifier（除非 lock 更新要求与现有 `^` 一致，无需改）。
- 窄门：`npm --prefix residential-monitor run typecheck` 若 lock 只动 types/ts-eslint；仍以 `just ci` 为批结束门。
- 若 `tauri-plugin-notification` 2.4.0 编译失败：对该 crate 钉回 2.3.3 后再 `just ci`。

### Batch 2 — GitHub Actions

- `.github/workflows/ci.yml`：`actions/checkout@v4` → `@v7`，`actions/setup-node@v4` → `@v7`。
- 保留 `persist-credentials: false`、Node 18/20/22 矩阵、monitor `node-version: "22"`、`dtolnay/rust-toolchain@stable`。
- 本机 `just ci` 仍要跑（证明未误改其它门）。托管 runner 证明在之后的 PR/push。

### Batch 3 — lucide-react 1

- `residential-monitor/package.json`：`lucide-react` → `^1.39.0`，再 `npm --prefix residential-monitor install`。
- 编译/类型失败时按缺失导出改 import，不改图标语义与 `data-*` 测试契约。
- 文件面：`src/**/*.tsx` 中现有 lucide import（约 18 处）。

### Batch 4 — Cargo Breaking（内部顺序 sha2 → rand → tungstenite）

- `sha2` `"0.10"` → `"0.11"`。保持 `Digest` + `Sha256::digest` / `Sha256::new` + `hex::encode` 小写 hex。
- `rand` `"0.9"` → `"0.10"`。只动 `candidate_schema.rs` 的 `SmallRng` / `Rng` / `SeedableRng`。
- `tokio-tungstenite` `"0.27"` → `"0.30"`。只动 `transport.rs` 测试夹具。
- 每改一个 crate 先 `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`；三个都过再 `just ci`。
- 某一个无法在少量补丁内编译：钉回该 crate 旧版本，其余已绿的保留。

### Batch 5 — eslint-plugin-react-hooks 7

- specifier `^5.2.0` → `^7.1.1`。留在 ESLint 9。
- 按包内实际 flat 导出改 `eslint.config.js`（当前为 `reactHooks.configs.recommended.rules`）。
- 不升 `eslint` / `@eslint/js` 到 10，不升 `eslint-plugin-react`。

### Batch 6 — TypeScript 6.0.3

- `typescript` → `^6.0.3`（不超过 6.0.x）。
- `npm --prefix residential-monitor run typecheck` 红则修类型，不关 `strict`。
- 不升到 7。

### Batch 7 — Vitest 4

- `vitest` → `^4.1.11`。保持 Vite 7。
- 按 Vitest 4 配置迁移 `vite.config.ts` 的 `test` 与 `tsconfig` 的 `types`。
- 不改测试断言语义。

### Batch 8 — Vite 8

- `vite` → `^8.2.2`。`@vitejs/plugin-react` 留 5.x。
- 核 `manualChunks` / `minify: "esbuild"` / `chunkSizeWarningLimit: 500` 仍有效。
- 构建块超 500 kB 不得只调高阈值。

### 收尾

- 重跑 `npm --prefix residential-monitor audit`。
- 再试 `cargo audit --file residential-monitor/src-tauri/Cargo.lock`；失败则记入任务 notes，不阻断已绿的 `just ci`。
- `CHANGELOG.md` Unreleased 英文一条，概括本轮实际落地的批次（跳过的写明）。
- 不把 docs 构建加入 `just ci`。

## 风险文件

| 文件 | 风险 | 回退 |
|---|---|---|
| `residential-monitor/package-lock.json` | 混入多批 | 每批单独还原 lock |
| `residential-monitor/src-tauri/Cargo.lock` | 同上 | 同上 |
| `residential-monitor/eslint.config.js` | hooks 7 导出形状 | 还原 + 留 hooks 5 |
| `residential-monitor/vite.config.ts` | Vitest 4 / Vite 8 配置 | 还原该批 specifier |
| `residential-monitor/src-tauri/src/**/*.rs` | sha2/rand/tungstenite API | 只回退对应 crate 与其源码 |
| `.github/workflows/ci.yml` | 矩阵或 credentials 被改 | 只允许两个 uses 行升主版本 |
| 根 `package.json` | 误加依赖或改 engines | 立即还原 |

## 验证命令

每批结束：

```text
just ci
```

Batch 4 内部额外：

```text
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
```

全任务结束额外：

```text
npm --prefix residential-monitor audit
cargo audit --file residential-monitor/src-tauri/Cargo.lock
```
