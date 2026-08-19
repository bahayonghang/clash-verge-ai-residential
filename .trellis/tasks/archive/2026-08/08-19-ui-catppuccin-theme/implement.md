# 实施计划：Catppuccin 主题

## 启动前门禁

- [ ] 用户已批准父任务规划摘要。
- [ ] 本子任务已 `task.py start`。
- [ ] 已读 frontend / backend checklist。

## 执行顺序

1. Rust `UiTheme` + 设置键 + Command + bootstrap + 回落测试。
2. 前端 `theme.ts` + 设置控件 + i18n 键。
3. CSS 四口味与按钮 / label 角色。
4. Latte 图标对比检查。

## 验证

- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`
- `npm --prefix residential-monitor run typecheck && npm --prefix residential-monitor run lint && npm --prefix residential-monitor test && npm --prefix residential-monitor run build`

## 回滚

删除 `ui_theme` 与 `data-theme`，恢复硬编码 `:root`。
