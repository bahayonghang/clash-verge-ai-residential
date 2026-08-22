# 实施计划：启动即监控与设置页现代化

## 启动前门禁

- [x] 父任务仍为 `planning`；用户以独立消息批准最新规划后才能 `task.py start`。
- [x] 重新读取 `08-19-monitor-tray-status` 当前任务 / 工作树状态；确认其 `lib.rs` 修改已落地或明确协调顺序。
- [x] 现有 23 项及之后的 unrelated dirty paths 已记录，实施只选择本父子任务拥有的文件。
- [x] `trellis-before-dev` 读取 residential-monitor frontend / backend 规范；UI 编辑前读取 `impeccable` craft-floor。

## 依赖顺序

1. 启动自动连接子任务：冻结 open/reconnect policy 与单实例激活 seam，完成 Rust / IPC 回归。
2. 设置页重构子任务：消费权威状态和命令，完成信息架构、视觉与可用性。
3. 父任务整合：在两个 child 都通过各自 gate 后做跨层回归、视觉复核、spec 更新和 closeout。

## 子任务 1：启动即自动连接与监控

- [x] 在 `AppFacade` / `DesktopRuntime` 建立纯的“已保存配置 + 正常 owner + open/reconnect”策略，首启无地址保持未配置。
- [x] 让 owner 冷启动、`--background` 与托盘打开窗口共享唯一 collector；若需要，补 Windows named activation event，使第二实例只激活 owner。
- [x] 复用 `reconnect_now` / `resume_collector`，禁止新增 Tokio interval、WebView timer、writer 或 probe-as-monitor 分支。
- [x] 增加无配置、有效配置、手动断开、重新打开、暂停 / 恢复、RecoveryOnly、第二实例不启动 collector 的测试。
- [x] 更新必要的中文状态 / action 文案，但不改 secret 脱敏与现有错误码。

## 子任务 2：设置与数据管理界面

- [x] 读取并保存 `SettingsSection` / `SettingsDraft`；拆分纯渲染函数，保持 secret 只通过 `input.value` 写入。
- [x] 建立二级导航与五个分组：外观与语言、连接与监控、数据与备份、关于、危险区域；默认连接。
- [x] 连接分组消费权威 health / collector 状态，区分单帧测试、持续监控、未配置与失败状态；接入保存、测试、重连、断开。
- [x] 主题卡片、语言分段、表单行、状态徽章、危险区和帮助文案沿用四套主题变量；更新中英 i18n。
- [x] 添加 settings-scoped CSS：桌面两栏、窄窗折叠 / 横向导航、40 px 命中区、focus、reduced motion、tabular numbers；不使用 `transition: all`。
- [x] 保留日志、备份、恢复、retention、物化汇总、VACUUM、关于和删除语义；Recovery 不渲染危险区。

## 验证与证据

- [x] `npm --prefix residential-monitor run typecheck`
- [x] `npm --prefix residential-monitor run lint`
- [x] `npm --prefix residential-monitor test`
- [x] `npm --prefix residential-monitor run build`
- [x] `cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check`
- [x] `cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
- [x] `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`
- [x] `git diff --check` 使用仓库约定的 CRLF 感知检查方式，避免 Windows 行尾误报。
- [x] 运行一次 `node C:\Users\lyh\.skillsmanage\skills\impeccable\scripts\detect.mjs --json`，仅对 changed UI targets；修复机械发现后不重复运行 detector。
- [ ] 通过 `just tdev` /实际 Tauri WebView 捕获 `desktop.png`（1200×800）与 `narrow.png`；验证文件有效且无黑屏 / 半加载（当前已有安装实例占用 single-instance）。
- [ ] 按 `impeccable` finish reviewer 要求独立复核截图、方向契约、键盘 / reduced-motion / error / loading 状态；截图缺失，已用 fresh in-thread degraded review 记录。

## 风险与回滚点

- `lib.rs` 与托盘子任务同文件：先协调当前基线，按函数合并；禁止 reset / checkout 整文件。
- 自动恢复若误将“普通重绘”视为打开，会覆盖手动断开；保留纯策略测试和手动断开 fixture。
- 设置页动态重绘可能清空草稿或泄漏 secret；先写 draft / `input.value` 测试，再扩展 CSS。
- 视觉证据若只能得到浏览器 fixture，标记桌面证据缺失，不把 fixture 当 Tauri 验收。

## 完成顺序

- [ ] 各 child 自己运行实现与检查，记录验证结果。
- [ ] 只有业务改动验证通过后才选择性提交；spec 更新使用独立语义提交。
- [ ] 父任务做最终集成检查，随后按 Trellis finish-work：work commit → archive child / parent → journal；不 push，除非另行授权。
