# 实施计划：托盘状态与左右键

## 启动前门禁

- [ ] 用户已批准本任务最新规划摘要。
- [ ] 已 `task.py start`，状态为 `in_progress`。
- [ ] 已读 `trellis-before-dev` 与 residential-monitor backend / frontend checklist。
- [ ] 未获确认前不跑 `just tinstall`、Credential Manager 真机写入或登录自启动写入。真机托盘走查用 `just tdev`。

## 执行顺序

### 1. 映射与测试

- 在 `c2/desktop.rs` 增加 `TrayVisual`、`TrayChrome` 与 `tray_chrome(running, session, storage_ok)`。
- 按 design 表驱动测试写入同一文件的 `#[cfg(test)]`。
- 可选：在 `DesktopRuntime` 增加上次已应用 visual / `collector_running`，供 `sync_tray_chrome` 跳过重复 `set_icon`。

**Gate 1**：映射测试覆盖 PRD 四态与暂停优先、存储故障 tooltip 键。

### 2. 四态托盘图标

- 从 `src-tauri/icons/icon.png` 派生四枚 PNG（画布与源图相同，512×512）：`tray-collecting.png`、`tray-connecting.png`、`tray-paused.png`、`tray-fault.png`。
- 保留白底房子标记；右下角色点带深色描边。不改 `icon.png` / `icon.ico` / `32x32.png` / `128x128.png`。
- 各 PNG 旁可留 `.json` prompt 记录，与现有 `icon.png.json` 一致。

**Gate 2**：四枚文件在仓库内，窗口图标资源未改。

### 3. Tauri 托盘接线

- `build_tray`：`show_menu_on_left_click(false)`；`on_tray_icon_event` 左键 Up 与左键 DoubleClick 走 `open_main_window`。
- `build_tray_menu` 接收 `collector_running`，暂停/继续 `enabled`。
- 抽出 `sync_tray_chrome`：设图标、tooltip、按需重建菜单。`include_image!` 嵌入四态 PNG。
- 在 collector tick 结束、托盘 pause/resume/reconnect、对应 command、`apply_locale_chrome`、以及会改 `session_status` 的设置/测试/断开入口调用 `sync_tray_chrome`。
- tooltip 用 `i18n::t` + `health_title`，格式见 PRD R3。

**Gate 3**：`cargo fmt --check`、`clippy -D warnings`、`cargo test --workspace` 通过。

### 4. 前端与文案

- 不改 `TraySummaryDto` 形状，除非编译迫使对齐。实时空态仍读 `collectorRunning`。
- 不为托盘新增前端 i18n 键；文案以 Rust `i18n.rs` 为准。

**Gate 4**：`npm --prefix residential-monitor` typecheck / lint / test 通过。

### 5. 真机走查

- `just tdev` 重启后看通知区（旧图标会缓存到重启）。
- 左键打开窗口且不弹菜单；右键菜单五项；暂停/继续互斥可点。
- 暂停 → 黄点 + 「采集已暂停」；恢复且已连接 → 绿点；断开/鉴权失败 → 红点。
- 任务栏窗口图标仍无色点。

**Gate 5**：AC1–AC6、AC8 在 Windows 通知区可观察。

## 验证命令

```
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
```

真机：`just tdev`。不要 `just tinstall`。

## 风险文件与回滚

- `residential-monitor/src-tauri/src/lib.rs`：托盘创建与事件，改动面最大。
- `residential-monitor/src-tauri/src/c2/desktop.rs`：映射与测试。
- `residential-monitor/src-tauri/icons/tray-*.png`：新增资源。
- 不要改 `icon.png` / `icon.ico`，否则任务栏产品标记会被色点污染。

回滚：删除 `sync_tray_chrome` 与四态 PNG，托盘回到单枚窗口图标 + 固定产品名 tooltip + 左键弹菜单。不碰数据库。

## `task.py start` 前检查

- [ ] `prd.md` 无阻塞开放问题，已做收敛。
- [ ] `design.md` 与 `implement.md` 已写。
- [ ] `implement.jsonl` / `check.jsonl` 已有真实 spec 条目。
- [ ] 用户已批准本规划摘要。
