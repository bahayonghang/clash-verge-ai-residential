# C5 证据索引

候选工作区：`dev` 上 C4 提交 `e83cc5c` 之后的 C5 工作树。本任务不自动发布 Release。

## 输入

| 来源 | 路径 | 状态 |
|---|---|---|
| C0 | `archive/2026-08/08-18-monitor-foundation-spike/research/` | 决策与 30 天生成证据存在。NSIS 基线资产缺失 |
| C1 | `archive/2026-08/08-18-monitor-collector-storage/research/c1-gate-status.md` | 自动门通过 |
| C2 | `archive/2026-08/08-18-monitor-desktop-realtime/research/c2-gate-status.md` | 自动门通过。安装态未跑 |
| C3 | `archive/2026-08/08-18-monitor-reporting-data/research/c3-gate-status.md` | 自动门通过。完整三档库未重跑 |
| C4 | `archive/2026-08/08-18-monitor-alerting-diagnostics/research/c4-gate-status.md` | 自动门通过。通知真机与 10k 完整组合未跑 |

## C5 命令

```text
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c5::
cargo run --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- c5-fault
cargo run --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- c5-concurrent
cargo run --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- c5-soak-smoke
cargo run --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- c5-supply
cargo run --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- c5-baseline
just monitor-c5-auto
just monitor-check
just ci
```

`c5-baseline` 在基线缺失时退出码 2。这是预期失败，不是用当前 installer 冒充旧版本。

## 影响矩阵

| 变化 | 必须重跑 |
|---|---|
| C1–C4 语义 / migration | 所属子任务 + C5 受影响 AC |
| 前端状态文案 / 无障碍 | C5-AC3 |
| 删除 / VACUUM | C5-AC7 / C5-AC12 |
| lockfile / capability / CSP | C5-AC13 |
| 完整库或 soak 参数 | C5-AC8 至 C5-AC11 |
