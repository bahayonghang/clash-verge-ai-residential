# 技术设计：Catppuccin 主题

边界、映射与回落见父任务 `08-19-ui-catppuccin-layout/design.md`。

## 本子任务落地

- 新增 `residential-monitor/src-tauri/src/theme.rs`（或放在 `i18n.rs` 旁的独立模块）：`UiTheme`、`THEME_SETTING_KEY`、`parse` / `as_str`。不要复用 `i18n::SETTING_KEY`。
- `AppFacade` 增加 `ui_theme`。`open` 读设置；`bootstrap` 写出；`save_ui_theme` 镜像 `save_ui_locale`。
- `lib.rs` 注册 `save_ui_theme`。
- 前端 `src/theme.ts`：`UiTheme` 类型、`parseUiTheme`、`applyTheme`。不要把解析散写在 `main.ts`。
- 设置页 `<select id="ui-theme">` 的 `change` 立即 `save_ui_theme` + `applyTheme` + `paint()`，不必等「保存设置」。
- 测试：Rust `ui_theme_persists_and_falls_back_to_mocha`，镜像 locale 测试。TS 解析非法值。

## 不改

连接查询、核算、托盘文案（托盘不因换肤改图标集，除非现有托盘图标在亮色下不可见；本任务不强制换托盘位图）。
