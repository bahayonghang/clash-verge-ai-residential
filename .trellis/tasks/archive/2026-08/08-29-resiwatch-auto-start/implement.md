# ResiWatch Windows 登录自启动实施计划

## Preconditions

- [x] 用户在本轮最终规划摘要之后，以新的消息明确批准实施。
- [x] `task.py start` 前确认任务仍为 `planning`、Git dirt 边界未变化、`implement.jsonl` 与 `check.jsonl` 均为真实条目。
- [x] 实施前加载 `trellis-before-dev`，读取 residential-monitor backend/frontend 规范。
- [x] 不运行 `just tinstall`、不写真实启动项、不注销/重启，除非用户对这些本机动作另行明确授权。

## 1. Rust plugin and production adapter

- [x] 在 `src-tauri/Cargo.toml`/`Cargo.lock` 增加官方 `tauri-plugin-autostart`，不增加前端 npm guest binding。
- [x] 在 `lib.rs` 初始化插件，参数唯一来自 `AUTOSTART_ARGUMENT`；初始化本身不得 enable。
- [x] 在 C2 desktop/system seam 实现 `TauriAutostartPort`；将 `FakeAutostart` 隔离到 `#[cfg(test)]`/测试模块，并提供 command core 可注入的 manager seam。
- [x] 从 `AppFacade` 结构体、正常 boot 与 recovery 构造移除 `FakeAutostart`。
- [x] 增加只使用 fake/mock 的 enable/disable/readback 与错误映射测试；自动测试不得实例化真实 plugin manager 或写 HKCU。确认 command 执行不长时间持有 facade mutex。

## 2. Rust commands and contract

- [x] 定义 `AutostartStateDto { enabled: bool }`。
- [x] 增加 `get_autostart_state` 与 `set_autostart_enabled` commands，并注册到 `generate_handler!`。
- [x] set command 走 `apply_autostart`，严格执行 write -> `is_enabled` readback。
- [x] 增加中英文 `autostart_unavailable`/重试文案；用含 executable path/注册表/平台原文的注入错误证明 IPC 与日志只记稳定 code/错误类。
- [x] 证明未新增 SQLite/UI preference 字段，也未向 capability 添加 `autostart:*` guest permission。

## 3. Frontend state and UI

- [x] 在 `dto.ts`/settings hook 增加严格 decoder 与独立 autostart request state/sequence。
- [x] 进入“连接与监控”时加载 OS 状态；`saving` 时跳过进入刷新，set 的最终 readback 拥有提交权；加载/保存期间禁止重复写入，失败保留最后确认状态。
- [x] 新建 `startup-section.tsx`，在现有分区增加标题、说明、Switch、状态、重试入口。
- [x] 开启动作加入无新依赖的内联 `role="alertdialog"` 确认区，完成标题/说明关联、Escape、确认/取消与焦点返回；取消不调用后端，关闭动作直接执行。
- [x] 补齐 zh/en 文案与键盘、focus、`aria-label`/checked/disabled 行为。

## 4. Regression tests

- [x] Rust 单测覆盖：默认无写入、fake enable/disable、写后回读、write/readback failure、脱敏、`--background`、后台窗口隐藏、single-instance 不重复 collector；测试进程不得写真实启动项。
- [x] Hook 测试覆盖：load、success、failure、保存中重进分区、stale response、快速重复操作、非 Tauri fallback。
- [x] 组件测试覆盖：开启确认/取消、关闭、loading/saving disabled、retry、ARIA 与中英文 copy。
- [x] 运行最小门：`npm --prefix residential-monitor test -- <focused tests>` 与相关 `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib <filter>`。

## 5. Docs and executable spec

- [x] 更新 `docs/install.md`、`docs/first-run.md`、`docs/upgrade-uninstall.md`、`docs/release-checklist.md`、`docs/known-limits.md`。
- [x] 将稳定的生产组合根、OS-truth、Rust-only capability 与安装态验证契约写入 `.trellis/spec/residential-monitor/backend/modules-and-errors.md` 和 frontend 相关 spec。
- [x] 检查文档不把未做的登录/重启验证写成已通过。

## 6. Quality gates

- [x] `git diff --check`
- [x] `npm --prefix residential-monitor run check`
- [x] `cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check`
- [x] `cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
- [x] `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`
- [x] `npm run check:secrets`
- [x] `just ci`
- [x] 由独立 Trellis check 核对 PRD/设计/测试/文档，无剩余生产 Fake 路径。

## 7. Approval-gated installed validation

- [x] 向用户说明安装和启动项写入动作，并取得明确授权。
- [x] 在真实注销/重新登录前说明会中断当前会话；用户自行完成两次登录验证并确认结果。
- [x] `just tinstall` 安装 current-user NSIS，确认路径为 `%LOCALAPPDATA%\ResiWatch`。
- [x] 开启后由命令核对 OS 启动项 executable 与唯一 `--background` 参数；设置页回读为开启由用户人工确认。
- [x] 用户完成一次真实 Windows 登录，确认无主窗口弹出、托盘可用、唯一 collector 恢复。第二实例契约由 AC6 自动化覆盖，本次人工登录报告不扩展到第二实例验证。
- [x] 设置关闭后回读 false，并在下一次登录确认不再启动。
- [x] 安装态命令采集证据与用户人工登录报告均已记录；AC9 已关闭，可进入提交和归档。

安装态证据与人工验证来源见 `evidence/installed-validation.md`。

## Risky files and rollback points

- `src-tauri/src/lib.rs`：plugin 初始化顺序、command handler 与 single-instance/后台启动交叉；先做 Rust focused tests。
- `src-tauri/src/c2/desktop.rs`、`facade.rs`：生产 seam 与 fake 移除；确保测试构造仍可注入 fake。
- `src/hooks/use-settings.ts`：当前多类请求共享状态；autostart 必须独立序号，避免与 secret/collector 请求相互取消。
- `components/features/settings/**`：保持现有五分区与滚动布局，不做无关设置页重构。
- 回滚最小单元：UI/commands/adapter/plugin dependency 一起撤销，保留既有 `--background` 生命周期。
