# 设计：侧栏色井、图表浮层、排名表头

## 边界

只改 `residential-monitor` 前端渲染与 `DESIGN.md` 导航色规则。不改 RouteId、IPC、DTO、查询、核算、下钻。侧栏宽度、密度、字号、主题键沿用现命令。

触及文件：

- `src/styles/globals.css`：`--popover` 抬升、`--nav-*`、`--nav-item-gap`
- `src/components/layout/sidebar.tsx` 与 `sidebar.test.tsx`
- `src/components/charts/rank-bar.tsx`、`share-donut.tsx`、`trend-area.tsx`（浮层外壳抽共用）
- `src/components/features/dimension/rank-table.tsx`、`rank-bar-card.tsx`
- `src/components/features/reports/ranking-table.tsx`
- 新增共用：`src/nav-tints.ts`（或放 `sidebar.tsx` 旁）、`src/components/charts/chart-hover.tsx`、`src/components/common/sortable-th.tsx`
- `DESIGN.md`：修订 One Blue Rule

不碰 `src/main.ts` / `src/styles.css` 旧壳。

## 1. 侧栏色井

未选中业务项：

```text
[ 色井 20–28px | 标题 ]
  井底 = color-mix(in srgb, var(--nav-tint) 18%, transparent)
  图标 = var(--nav-tint)
  标题 = text-muted-foreground → hover:text-foreground
```

选中业务项：整行 `bg-primary text-primary-foreground`。井底 `color-mix(in srgb, white 18%, transparent)`，图标与标题白色。

关于 / 设置：无 `--nav-tint`，无色井，图标与标题同色。

`color-mix` 直接吃 CSS 变量。禁止 `color + "15"` 这种只对 hex 生效的拼串（口径卡可以，因为传入的是 `#3B82F6`）。

### Token

四主题都声明 `--nav-overview` … `--nav-alerts`。图表已有的走 `--chart-*` / `--destructive`，缺的补齐：

| 路由 | Token | Mocha / Macchiato / Frappé | Latte |
|---|---|---|---|
| overview | `--nav-overview` | `--chart-1` `#3b82f6` | `--chart-1` `#0063ff` |
| live | `--nav-live` | `--chart-3` `#06b6d4` | `--chart-3` `#00c7ff` |
| residential | `--nav-residential` | `--chart-5` `#f59e0b` | `--chart-5` `#ff8c42` |
| host | `--nav-host` | `--chart-2` `#8b5cf6` | `--chart-2` `#7b61ff` |
| rule | `--nav-rule` | `--chart-4` `#10b981` | `--chart-4` `#00d084` |
| chain | `--nav-chain` | `#0ea5e9` | `#0284c7` |
| process | `--nav-process` | `#f97316` | `#ea580c` |
| reports | `--nav-reports` | `#6366f1` | `#4f46e5` |
| alerts | `--nav-alerts` | `--destructive` `#ef4444` | `#ef4444` |

映射表只在一处（`nav-tints.ts` 或 sidebar 常量），值为 `var(--nav-overview)` 这种引用。测试断言每条业务按钮带 `data-nav-tint="<route>"` 或 style 里的 `--nav-tint`。

### 间距

`nav` 从 `space-y-1` 改为 `flex flex-col gap-[var(--nav-item-gap)]`。

| 密度 | `--nav-item-gap` | `--nav-item-py` |
|---|---|---|
| comfortable | `0.5rem` | 保持 `0.75rem` |
| compact | `0.25rem` | 保持 `0.32rem` |

底栏关于 / 设置仍用较小 gap（可维持 `space-y-1`），避免把设置顶出 800px 窗口。

井尺寸：comfortable `size-7`，compact `size-6`；图标 `size-4`。220px 英文单行 + `truncate` 契约不变。

## 2. 浮层与 popover

### Token

暗色 `--popover` 必须高于 `--card`，`--popover-foreground` 保持浅色，边用实色而不是与卡片相同的 8% 白。

| 主题 | `--card`（现状） | `--popover`（目标） |
|---|---|---|
| latte | `#ffffff` | `#ffffff`（加阴影即可） |
| frappe | `#252a38` | `#32384a` |
| macchiato | `#1e2331` | `#2a3042` |
| mocha | `#171c2b` | `#242a3c` |

Radix `TooltipContent` 已是 `bg-popover`，跟 token 走，不必分叉。下拉菜单同样抬升，这是预期。

### RankBar 浮层

抽出 `ChartHover`（`components/charts/chart-hover.tsx`）：

- 外壳：`rounded-lg border border-border bg-popover p-3 text-popover-foreground shadow-lg`
- 标题：条目 `label`
- 一行数值：`valueFormatter(value)`，不渲染系列名
- Recharts：`<Tooltip content={<RankHover locale formatter />} cursor={false} isAnimationActive={false} />`

`TrendTooltip` 改为包一层 `ChartHover` 外壳，避免三套边框。`ShareDonut` 仅在 `onHover` 为空时挂该浮层；有 inspect 回调时保持现状（不叠两层）。

Y 轴：`tick={{ fontSize: 11, fill: "var(--muted-foreground)" }}`。

## 3. 排名表头

抽出 `SortableTh`：

```text
<th aria-sort={...} class="bg 随 thead">
  <button class="inline-flex items-center gap-1 font-semibold">
    列名
    <Icon aria-hidden class="size-3.5 shrink-0">
      none → ChevronsUpDown text-muted-foreground/80
      descending → ChevronDown text-foreground
      ascending → ChevronUp text-foreground
    </Icon>
  </button>
</th>
```

`RankTable` thead：`bg-muted/40 text-foreground`，可排序列走 `SortableTh`，不可排序列纯文本。进入页面 `aria-sort="descending"` 在「下行」。

报告 `RankingTable` 的 `head()` 换成同一组件。实时表 Unicode 三角不迁。

归因：删除 `RankTable` 里的 `<AttributionQualityNote />`。`RankBarCard` 保留。`rank-table.test.tsx` 增加「渲染结果不含 字段归因」。

## 4. DESIGN.md 修订

Named Rules 改为：

- 选中导航与主按钮仍只用主色蓝。
- 未选中业务导航用 `--nav-*` 色井认页。色井不是状态徽章，不表示健康/告警严重度（告警路由的红只是身份色）。
- 关于 / 设置无色井。

Navigation 组件段补色井尺寸与选中白图标。Data table 段补：可排序列表头必须有可见 lucide 方向图标，默认排序列在进入时就带着图标。

## 5. 兼容与回滚

- 旧偏好键不变。非法主题仍回落 Mocha，新 token 写在四段 `:root[data-theme]` 内。
- 回滚：还原上述文件与 `DESIGN.md`。色井与浮层无持久化，无迁移。
- `prefers-contrast: more` 现有块会把边和 muted 拉深；色井仍靠 `color-mix`，对比块不必为九色各写一套。
- `prefers-reduced-motion`：本任务不加新动画。

## 6. 取舍

| 方案 | 结论 |
|---|---|
| 只改图标 `currentColor` | 用户已否。九项紧排时不够认。 |
| 选中行也换路由色底 | 用户已否。扣掉单一蓝认页。 |
| 全局抬升 `--popover` vs 只给图表写死底 | 抬升 token。Radix 浮层同一缺陷，一处修。 |
| 报告扇形图也强制 Recharts 浮层 | 不。inspect hover 已承担读数，叠两层会挡。 |
| 实时表也换 lucide 三角 | 不。列宽手柄与查询排序合同不同，越出截图范围。 |
