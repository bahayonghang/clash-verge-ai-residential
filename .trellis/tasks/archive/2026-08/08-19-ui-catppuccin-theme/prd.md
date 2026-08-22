# Catppuccin 明暗主题

## Goal

用户在设置里选择 Catppuccin 口味后，整个窗口（侧栏、五页、Recovery）立即换成该口味的完整构图，重启后保持。

## Background

父任务：`08-19-ui-catppuccin-layout`。本子任务先于概览与筛选，因为后两者要消费语义 token 与 `.btn-secondary`。

当前 `styles.css` 硬编码海军 / 浅灰双色壳。语言已有 `ui_locale` 持久化模式可复制。

## Requirements

- 设置页在 `#ui-locale` 旁增加 `#ui-theme`：Latte、Frappé、Macchiato、Mocha。默认 Mocha。存储值 `latte|frappe|macchiato|mocha`。
- `save_ui_theme` + `put_setting("ui_theme")`。非法或缺失回落 Mocha。Recovery 无库时只改内存。
- `BootstrapDto.uiTheme` 可选。前端缺字段按 Mocha。
- `html[data-theme]` 驱动本地 CSS 变量。Latte 的 `color-scheme: light`，其余 `dark`。每个口味独立构图。
- 主按钮 / 次要按钮角色分离。全局堆叠 `label` 改为 `.stack`，为筛选工具条让路。
- 无 CDN、无远程字体、无 npm 主题包。`prefers-contrast: more` 与 `:focus-visible` 覆盖当前口味。
- 本地导航图标在四口味下可辨认。
- 中英文标签。口味名为专有名词。不改口号与删除确认短语。

## Out of Scope

- 概览口径重组、实时筛选 markup（后续子任务）。
- 跟随系统浅色/深色。
- 改核算、连接查询、route。

## Acceptance Criteria

- [ ] 设置可切换四口味，写入本机设置；缺省与非法值回落 Mocha。
- [ ] 切换后侧栏、当前页、Recovery 立即换肤；重启后保持。
- [ ] 构建产物无远程主题资源。
- [ ] Latte 与三个暗色都不是简单反相；导航选中与主按钮仍是该口味的 blue。
- [ ] 相关 Rust 测试与 `npm --prefix residential-monitor` typecheck / lint / test / build 通过。

## Key Decisions

- 持久化复制 `ui_locale`，不把主题塞进控制器 JSON。
- 默认 Mocha，保留夜间工作台。
