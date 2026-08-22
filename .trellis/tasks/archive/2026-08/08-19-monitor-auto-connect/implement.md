# 实施计划：启动即自动连接与监控

## 前置门禁

- [x] 用户批准父任务最终规划后，才 `task.py start`；本子任务不得单独越过父任务批准。
- [x] 读取 `08-19-monitor-tray-status` 实际改动与验证状态；`lib.rs` 以其最新基线合并。
- [x] 执行 `trellis-before-dev`，加载 backend / frontend 契约；不改 unrelated dirty paths。

## 步骤

1. [x] 在 `DesktopRuntime` / `AppFacade` 增加纯的 open/reconnect policy，区分 cold boot、window reopen、manual disconnect、paused、no address 与 RecoveryOnly。
2. [x] 把 tray / menu / owner activation 的打开动作接到统一 policy；第二实例只发送激活，不进入 Tauri builder、不启动 collector。
3. [x] 若缺少 Windows owner 激活通道，使用现有 `windows-sys` Threading 能力实现 named event；不得引入 Tauri single-instance plugin 或新 runtime dependency。
4. [x] 保持 `collector_loop_tick` 唯一、`test_controller` 单帧语义和 existing Channel bootstrap；补中文状态 action（若证据表明已有键足够则不添加重复文案）。
5. [x] 添加纯状态 / fixture regression：有效持久地址、无地址、manual disconnect、tray reopen、background owner、RecoveryOnly、second instance。

## 验证

- [x] `cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check`
- [x] `cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
- [x] `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`
- [x] `npm --prefix residential-monitor run typecheck`
- [x] `npm --prefix residential-monitor run lint`
- [x] `npm --prefix residential-monitor test`
- [x] `git diff --check` 使用 CRLF 感知方式。
- [x] Windows installed / real controller / Credential Manager 未执行的证据保持 `UNVERIFIED`，不以单测代替。

## 回滚点

- policy / activation listener 可单独删除，恢复已有显式 reconnect。
- 不删除或迁移数据库，不清理凭据，不改 collector 核算接口。
