# C0 决策与验收状态

生成时间：2026-08-18。本文件只记录已执行命令和未执行项。未完成项不得标完成。

## 命令面

- `npm --prefix residential-monitor run check`：typecheck / lint / vitest / vite build 已通过。
- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`：15 通过，1 忽略（Windows Credential Manager 真机写入）。
- `cargo clippy ... -- -D warnings`：通过。
- `cargo fmt --check`：由质量门执行。
- 完整 `just monitor-check` 与根 `just ci` 在本轮末执行。

## 决策

| 主题 | 结论 | 证据 | 批准状态 |
|---|---|---|---|
| SQLite binding rusqlite 0.40 bundled | adopt；实测 sqlite_version 3.53.2，高于 3.51.3 | `.trellis/tasks/08-18-monitor-foundation-spike/research/evidence/sqlite-binding.json` | 待用户确认 |
| CredentialStore port + Fake / 进程内临时 | adopt 接口；Windows adapter 留给 C2 | `credential_port` 测试 | 接口可给 C1 fake resolver |
| Windows Credential Manager 真机 | 未跑 | `credential_windows` 为 ignored | 暂停，避免写入本机凭据 |
| TCP fixture secret 三态 | adopt 为受支持路径 | `transport_fixture` 测试 | 待用户审阅 |
| named pipe | best-effort，不发送 secret，TCP fallback | `controller_profiles` 测试 | 真机 Clash Verge 矩阵未跑 |
| NSIS / 托盘 / 自启动 / 通知 | 配置已写 current-user，未安装 | `tauri.conf.json` | 暂停，避免改本机安装态 |
| A=50 完整 30 天库 | 已实测；行数与期望一致；24.7s；约 221 MB | `research/evidence/generate-a50-d30.json` | 待用户确认 |
| A=250 完整 30 天库 | 已实测；行数与期望一致；481s；约 1.13 GB | `research/evidence/generate-a250-d30.json` | 待用户确认 |
| A=1000 完整 30 天库 | 已实测；行数与期望一致；3904s；约 4.35 GB | `research/evidence/generate-a1000-d30.json` | 用户已批准 |
| 10k / 1Hz / 30m 峰值 | 1800 帧；p50 15ms / p95 37ms / p99 3037ms / max 3321ms；零未解释丢帧；FULL | `research/evidence/peak-10k-30m.json` | 用户已批准；max 超过 3s 作为 C1 backpressure 约束 |

## C1 放行

用户已于 2026-08-18 批准 C0 并启动 C1。C1 输入清单见 `08-18-monitor-collector-storage/research/c1-input-from-c0.md`。

仍未做、也不阻塞 C1 启动的项：

- Clash Verge 真机 named pipe 矩阵
- 本机 NSIS 安装、登录自启动、系统通知
- 24 小时 soak
