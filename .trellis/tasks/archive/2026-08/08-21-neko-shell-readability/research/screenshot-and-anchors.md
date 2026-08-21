# 规则页截图与实现锚点

来源：用户 Image #1（Mocha、近 24 小时、Top 20 规则页）+ `refactor/neko-ui-port` 源码。实施时以本文件为证据索引，源码以仓库当前行为准。

## 侧栏

- `residential-monitor/src/components/layout/sidebar.tsx`：业务导航 `space-y-1`，未选中 `text-muted-foreground`，选中 `bg-primary`，无色井。底栏关于 / 设置同样无分色。
- `residential-monitor/src/styles/globals.css`：`--nav-item-py` comfortable `0.75rem` / compact `0.32rem`。暗色 `--popover` 与 `--card` 同值（Mocha `#171c2b`）。
- 色井先例：`caliber-card.tsx`、`stat-card.tsx` 的 `h-8 w-8` + 色值 15% 透明。那些卡片传入 hex，不能把 `"15"` 拼到 CSS 变量上；侧栏必须用 `color-mix`。
- 英文 220px compact 契约：归档任务 `08-21-en-sidebar-layout`；测试在 `sidebar.test.tsx`。

## 条形图浮层

- `charts/rank-bar.tsx`：Recharts 默认 `Tooltip`，`dataKey="value"` → 界面英文 `value :`。`contentStyle` 只用 `--popover`，未设文字色。Y 轴 `fill: "#888888"`。
- 共用：`dimension/rank-bar-card.tsx`、`dimension/drilldown-panel.tsx`、`residential/aggregate-section.tsx`。
- 可读先例：`charts/trend-area.tsx` 的 `TrendTooltip`。
- `charts/share-donut.tsx`：报告页 `onHover` 为真时不渲染 Recharts 浮层；无 `onHover` 时仍是默认浮层。

## 排名表

- `dimension/rank-table.tsx`：可排序名称 / 上行 / 下行 / 连接；默认 download 降序；`aria-sort` 有、图标无。`AttributionQualityNote` 与 `rank-bar-card.tsx` 重复。
- `reports/ranking-table.tsx`：thead 已有 `bg-muted/40`，排序无图标。
- 实时表 `live-table-sort.ts` 已有 Unicode 三角，本任务不改。

## 产品决定

- 未选中业务导航：图标色井。
- 选中：整行主色蓝，白图标浅白井。
- 关于 / 设置：无色井。
