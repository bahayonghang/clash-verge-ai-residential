# 实施计划：Catppuccin 主题、概览口径、实时筛选工具条

## 启动前门禁

- [ ] 用户已批准本规划摘要。
- [ ] 已读 frontend / backend spec checklist。
- [ ] 不在父任务上 `task.py start`。先启动 `08-19-ui-catppuccin-theme`。

## 执行顺序

### 1. `08-19-ui-catppuccin-theme`

- Rust：`UiTheme`、`put_setting("ui_theme")`、`save_ui_theme`、bootstrap 字段、回落 Mocha。
- 前端：`parseUiTheme`、`applyTheme`、设置页 `#ui-theme`、中英文标签。
- CSS：四口味映射、`.btn-secondary`、`label.stack` / `label.inline`、去掉 `.workspace` 写死 light。
- 图标在 Latte 侧栏可辨认。

**Gate**：非法值 Mocha；重启保持；无存储时内存切换不炸 Recovery；typecheck / 相关 Rust 测试。

### 2. `08-19-overview-caliber-layout` 与 `08-19-live-filter-toolbar`

主题合并后再开。二者无互相依赖，可顺次在同一工作区完成。

概览：成对口径、3 列网格、分类表、状态区。

筛选：工具条 markup、横向开关、次要「添加条件」、表格吃高。不改 `liveQuery` 形状。

**Gate**：宽窗无 7+1 孤儿卡；筛选查询与改前一致；语言 / 空态 / 删除确认回归绿。

### 3. 父任务集成

- 五页与 Recovery 在四口味下可扫。
- 回写 DESIGN.md / 表面 brief（实施完成后）。
- 父任务验收 AC1–AC5。

## 验证

- `npm --prefix residential-monitor run typecheck`
- `npm --prefix residential-monitor run lint`
- `npm --prefix residential-monitor test`
- `npm --prefix residential-monitor run build`
- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`（主题持久化与 bootstrap）
- 不跑 `tinstall`

## 风险文件

- `residential-monitor/src/styles.css` — token 与 label 选择器
- `residential-monitor/src/main.ts` — 概览 / 实时 / 设置渲染
- `residential-monitor/src/i18n/zh.ts`、`en.ts`
- `residential-monitor/src/dto.ts` — 可选 `uiTheme`
- `residential-monitor/src-tauri/src/c2/facade.rs` — bootstrap / save
- `residential-monitor/src/assets/icons/*` — Latte 对比

## 回滚点

1. 撤回 `ui_theme` Command 与 CSS 口味块。
2. 撤回 `renderOverview` / `renderLive` markup。
3. 查询与核算文件不应出现在 diff 里。
