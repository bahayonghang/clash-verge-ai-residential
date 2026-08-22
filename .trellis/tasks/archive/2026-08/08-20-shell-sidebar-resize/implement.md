# 实施计划：应用壳侧栏可调宽度

## 有序清单

- [ ] 关于页子任务已合入或至少其 diff 已稳定。`task.py start 08-20-shell-sidebar-resize`。
- [ ] 新增 `src/shell-width.ts` 与测试：DEFAULT/MIN/MAX、parse、clamp。
- [ ] Rust：设置键、`save_ui_sidebar_width`、`BootstrapDto.ui_sidebar_width`、persist/回落/无库测试。注册 Tauri 命令。
- [ ] `main.ts`：boot 应用 `--shell-width`；壳上挂 separator；pointer/keyboard；`shellDragging` 挡住 `paint`；与 `liveTableDragging` 互斥。
- [ ] `styles.css`：`.shell` 改用变量；手柄样式；reduced-motion。
- [ ] i18n：手柄 aria 文案。
- [ ] `npm --prefix residential-monitor run typecheck && lint && test && build`。
- [ ] `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib` 覆盖新 persist 测试；`cargo fmt --check`。

## 验证命令

```powershell
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml -- --check
```

## 回滚

删除键与命令，`.shell` 回到 `13.75rem`。前端忽略未知 Bootstrap 字段。
