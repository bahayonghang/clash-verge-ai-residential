# residential-monitor frontend

React 19 + Tailwind v4 + Vite 桌面壳。`components/**` 不得直接 `invoke`，IPC 只在 `hooks/**`。每个 hook 必须请求序号递增、过期响应丢弃、失败保留上次结果并单独暴露 `errorZh`。根仓库 frontend spec 只覆盖可粘贴 Clash 扩展，不适用于本子项目。十段 `RouteId`：`overview`、`live`、`residential`、`host`、`rule`、`chain`、`process`、`reports`、`alerts`、`settings-data`。入口是 `src/main.tsx`，禁止再引入 `src/main.ts` 或 `src/styles.css`。

## Pre-Development Checklist

- 读 `dto-and-decoding.md`：Channel / Command 载荷在边界解码。
- 读 `view-state.md`：前端只保存视图选择和 DTO 缓存。
- 改侧栏、图表悬停或排名表头时读根目录 `DESIGN.md` 的 Navigation / Data table，以及下面 Surfaces。
- 禁止 `window.__TAURI__`、eval、远程 URL 和 CDN。

## Surfaces

- 暗色三档 `--popover` 必须高于 `--card`。Recharts 默认 Tooltip 和 Radix 浮层都吃 `--popover`；同色时暗色读数消失。图表悬停用 `components/charts/chart-hover.tsx` 的 `ChartHover`。禁止只改 `contentStyle` 背景、留下英文系列名 `value :`。
- 业务导航色井用 `color-mix(in srgb, var(--nav-tint) 18%, transparent)`。禁止把 `"15"` 拼到 CSS 变量上（只对 hex 生效）。选中项仍是 `--primary` 整行白图标。关于 / 设置无色井。
- 可排序排名表走 `components/common/sortable-th.tsx`（lucide 方向图标 + `aria-sort`）。默认排序列进入页面时就要带着图标。实时表 Unicode 三角不迁到这套表头。
- 新表格一律走 `components/common/data-table.tsx`（`dataTableClasses` + `DataTableTh/Td/EmptyRow`）：数值列右对齐 + `tabular-nums`、表头 `text-xs` 与正文拉开档差、行 `hover:bg-muted/40`、容器不裸用 `w-full` 拉伸。排序状态仍由调用方持有，DataTable 不吞交互逻辑。≤3 行的小表格优先改紧凑读数行而非全宽表。家宽页排名表行首色点吃 `rank-bar.tsx` 导出的 `CHART_COLORS`，与图表同序。
- `StatCard` 图标色一律用 `colorToken`（1–5 映射 `var(--chart-N)`，背景 `color-mix(in srgb, var(--chart-N) 8%, transparent)`）。hex `color` 入参是遗留兼容路径，新调用禁止再写硬编码 hex。
- 页面标题层级约定：页标题 `text-xl font-semibold`、区块 h2 `text-base font-semibold`、子区块 h3 `text-sm font-semibold`；说明文字统一 `text-xs text-muted-foreground/80` 并收拢到标题旁一处 note 区。
- DOM 重构时 `data-state` / `data-identity` / `data-bucket-utc` / `data-caliber` / `data-share-kind` / `data-sort-icon` 与 `role` / `aria-*` 是测试契约：必须原样保留或同步更新测试断言；标题层级变化时检查 `getByRole("heading", { level })`。

## Quality Check

- `npm --prefix residential-monitor run typecheck`
- `npm --prefix residential-monitor run lint`
- `npm --prefix residential-monitor test`
- `npm --prefix residential-monitor run build`
- 关于页不得把未签名候选标成 `signed`。删除部分失败不得显示「已全部删除」。
