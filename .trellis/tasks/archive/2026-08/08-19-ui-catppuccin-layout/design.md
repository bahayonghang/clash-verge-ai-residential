# 技术设计：Catppuccin 主题、概览口径、实时筛选工具条

## Architecture

```text
设置页 #ui-theme
        │
        ▼
save_ui_theme(flavor)
        │
        ▼
put_setting("ui_theme", latte|frappe|macchiato|mocha)
AppFacade.ui_theme
BootstrapDto.uiTheme
        │
        ▼
<html data-theme="mocha" style="color-scheme: dark">
styles.css 按 data-theme 映射语义 token
renderOverview / renderLive / 其余页只消费 token
```

工作分三层，互不改对方合同：

1. 主题：Rust 设置键 + 前端 token 与按钮角色。
2. 概览：`renderOverview` 结构。只读 `LiveOverview` 已有字段。
3. 筛选：`renderLive` 工具条 DOM 与 CSS。`liveQuery` 与 `query_live_connections` 形状不变。

## Theme contract

新增 `UiTheme { Latte, Frappe, Macchiato, Mocha }`，默认 `Mocha`。

- 设置键：`ui_theme`。不要写进控制器 JSON。
- `save_ui_theme(raw) -> UiTheme`：非法值回落 Mocha。无 `storage`（Recovery）只改内存。
- `BootstrapDto.ui_theme`。前端缺字段按 Mocha。
- 前端 `parseUiTheme` + `applyTheme`：写 `document.documentElement.dataset.theme` 与 `color-scheme`（Latte=`light`，其余=`dark`）。
- CSS：`:root` 提供语义变量；`html[data-theme="…"]` 写入该口味的 Catppuccin 官方色。页面规则继续用 `--sidebar`、`--main`、`--card`、`--ink`、`--muted`、`--accent`、`--accent-pressed`、`--focus`、`--danger`、`--ok`、`--table-*`。新增 `--btn-secondary-bg`、`--btn-secondary-ink`、`--btn-secondary-border`。
- `.workspace` 不再写死 `color-scheme: light`。
- 按钮：默认主按钮用 `--accent`；`.btn-secondary` 用于添加条件、删除条件、次要设置动作中需要的项。行内「关闭」保持主按钮。
- 全局 `label` 改为 `.field`（或 `label.stack`）。设置 / 报告 / 告警的堆叠字段改用该类。工具条与勾选使用横向 `label.inline`。

## Catppuccin mapping

官方 palette 本地抄入 CSS，不请求 githubusercontent 或 npm。角色映射：

| 语义 | Latte | 暗色三口味 |
|---|---|---|
| `--sidebar` | mantle | mantle |
| `--sidebar-mid` | surface0 | surface0 |
| `--sidebar-text` | text | text |
| `--sidebar-muted` | subtext0 | subtext0 |
| `--main` | base | base |
| `--card` | crust | surface0 |
| `--ink` | text | text |
| `--muted` | subtext0 | subtext0 |
| `--accent` | blue | blue |
| `--table-head` | surface1 | crust |
| `--table-row` | mantle | surface0 |
| `--table-text` | text | text |
| `--ok` | green | green |
| `--danger` | red | red |
| `--focus` | lavender | lavender |

暗色三口味各自使用该口味的同名色，不共用 Mocha 值。Recovery 提示用 yellow，不占用 `--accent`。

## Overview structure

```text
section.overview
  section.caliber-grid
    article.caliber[meter | attributed | other | gap | over]
      h3 口径名
      dl 上行 / 下行
    article.caliber.session
      活跃连接
      覆盖
      健康
  section.panel.categories
    table 名称 / 上行 / 下行
```

- `.caliber-grid`：`repeat(3, 1fr)`（窄于约 52rem 时 2 列，再窄 1 列）。不再 `auto-fit` 孤儿卡。
- 分类键 = `Object.keys({ ...categoryUpload, ...categoryDownload })`。缺一侧写「未知」。
- `.workspace` / `.overview` 纵向 flex：分类表所在 panel 可 `flex: 1`，空态仍是一行，不把灰底留在卡片外。
- 不调用 `query_live_connections`。

## Live toolbar structure

```text
section.live-page
  header.live-toolbar
    p.status + 采样时间
    .live-filter-bar
      label.inline 只看家宽
      .filter-clauses
        .filter-row × N
      button#live-add-clause.btn-secondary
  .live-table-wrap
    table.data.live-table
```

`liveQuery` 变更路径（checkbox / field / mode / value / add / remove）保持现有事件委托。只改 markup 与 class。

## Compatibility

- `schemaVersion` 仍为 1。`uiTheme` 缺省解码，旧 bootstrap 仍可用。
- 打印：侧栏与按钮继续隐藏。打印背景跟当前口味或白底均可，不引入新路由。
- CSP：无远程样式。图标仍是 `img-src 'self'`。
- 动态重绘后仍按元素 `id` 恢复焦点。`#ui-theme` 加入该集合。

## Trade-offs

- 外观走后端设置键而不是仅 localStorage：与语言一致，重启与多 WebView 会话同一来源。代价是 `BootstrapDto` 多一个可选字段和一条 Command。
- 整窗跟随口味、放弃深浅双色壳：Catppuccin 桌面应用的标准读法。代价是现有 DESIGN.md 双色世界被替换；实施后由 Impeccable documenter 回写。
- 概览不复用实时表：避免两页抢同一查询游标，也避免把监控首页做成第二张连接表。

## Rollback

- 主题：删除 `ui_theme` 键与 `data-theme`，恢复 `:root` 硬编码。
- 概览：恢复 8 卡网格。
- 筛选：恢复当前 `live-filters` markup。查询层不动，回滚不触及 Rust 筛选。
