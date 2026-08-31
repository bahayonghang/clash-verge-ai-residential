# 家宽页面排版与表格样式重构 — 技术设计

## 边界

**可动**（样式与 DOM 结构层）：

- `residential-monitor/src/components/features/residential/`：index.tsx、monitor-section.tsx、aggregate-section.tsx、share-readout.tsx、report-section.tsx、caliber-note.tsx
- `residential-monitor/src/components/common/`：stat-card.tsx、overview-card.tsx、sortable-th.tsx（仅增量增强，向后兼容）
- 新增共享表格模块（见 C2）
- `residential-monitor/src/components/charts/rank-bar.tsx`（标签可读性、色序暴露）
- 对应测试文件（断言随 DOM 结构调整同步更新）
- `residential-monitor/src/components/features/overview/category-table.tsx`（可选接入共享表格，非强制）

**不可动**：

- `src/hooks/`、`src/ipc/`、`src/format/`、`src/dto.ts`、`src/lib/`：数据流与核算逻辑
- i18n key 与文案内容（`src/i18n/zh.ts` / `en.ts` 文字不动；仅当布局需要新增「分组标签」类文案时才允许新增 key，需用户确认）
- 交互行为：排序点击、direction/TopN 切换、运行报告、导出、路由跳转

## 测试钩子保护线（硬约束）

现有测试大量依赖 DOM 钩子，重构必须保留或同步更新测试：

- `data-state`（monitor-section 健康态、aggregate 状态、report-section 状态）
- `data-identity`（排名表行）、`data-bucket-utc`（趋势表行）、`data-caliber`（CaliberNote）
- `data-share-kind`（share-readout note）、`data-sort-icon`（SortableTh）
- `role="status"` / `role="alert"` / `aria-sort` / `aria-pressed` / `aria-labelledby` 及对应 id
- 标题层级变化（h2→h3 等）时检查测试里 `getByRole("heading", { level })` 类断言

## 组件契约变更

### C1 StatCard 颜色令牌化

现状：`color: string` 接收硬编码 hex，用 `${color}15` 拼 8% 透明度背景——对 `var(--chart-1)` 无效。

设计：新增可选 prop `colorToken?: 1|2|3|4|5`（映射 `var(--chart-N)`），背景改用 `color-mix(in srgb, var(--chart-N) 8%, transparent)`。保留 `color` hex 入参以兼容既有调用方；家宽页全部迁移到 `colorToken`。四主题（Latte/Frappé/Macchiato/Mocha）下 chart 令牌均有定义，天然正确。

### C2 共享表格规格收敛

新增 `residential-monitor/src/components/common/data-table.tsx`，导出统一 class 常量与轻量封装：

- 表容器：不再无脑 `w-full`；`w-auto min-w-0 max-w-full`（或 `w-full table-fixed` 仅在确需均分时），数值列 `text-right tabular-nums whitespace-nowrap`，名称列 `pr-6` 收敛列距
- 表头：`text-xs font-medium text-muted-foreground`（与正文 text-sm 拉开档差）
- 行：`border-b border-border/40 last:border-0 hover:bg-muted/40 transition-colors`（hover 统一，与 TrendTable 现状对齐）
- 空态行、loading 行的固定写法

形态：class 常量 + `DataTableTh`/`DataTableTd` 小封装即可，不做带排序逻辑的大组件——排序状态仍由调用方持有（功能不动）。`SortableTh` 内部对齐新表头档差（增可选 prop 而非改默认，避免外溢）。概览 `category-table.tsx` 接入作为顺手项。

### C3 家宽页层级与卡片分组

保持单列与区块顺序，结构调整为：

1. **页头**：页标题升到 `text-xl font-semibold`（与 DESIGN.md title 档一致）；口径总说明保留为页头下一行 note。
2. **实时监控**（h2 档，示例 `text-base font-semibold`）：CaliberNote + 采样时间收进标题行右侧/下方的固定 note 区；三 StatCard → HotspotCards → 占用读数。占用表当前常 1 行，改为紧凑读数行（名称 + 上行/下行数值组），不再用全宽表；数据多于阈值（如 >3 行）时仍用 C2 表格。
3. **家宽流量统计**（h2）：`ShareReadout` 的 h2 降为 h3 从属；占比 2 行表改为紧凑定义行（名称列与数值列靠拢）；3 张 StatCard 保留；named SQL 行降权为 note 区文字。排名与趋势各自保持 `OverviewCard` 容器。
4. **生成报告**（h2）：操作行（Switch + 运行按钮）、状态文字、CoveragePanel/CapabilityPanel/ExportPanel 装入卡片容器（复用 `OverviewCard` 或 `ui/card`）；状态文字（「尚未运行报告」）固定为卡片内状态区，用 `role="status"` 样式而非正文段落。

说明文字统一规格：`text-xs text-muted-foreground/80`，每区块至多集中在标题旁的一处 note 区，不分散在内容体中间。

### C4 排名区块

- 两组切换器（direction / TopN）加各自分组容器与 `role="group"` + `aria-label`，视觉间距拉开。
- 图表标签：`rank-bar.tsx` 的 `fontSize: 11` 与 `ellipsizeLabel` 截断策略优化——`rankAxisWidth` 上限放宽、`maxChars` 计算复核；截断时 `title`/Tooltip 已有完整值兜底（功能已有，不动）。
- 图表-表格对应：表格行首加 4px 色点 `var(--chart-N)`，`N = index % 5`，与 `CHART_COLORS` 同序（把 `CHART_COLORS` 从 rank-bar.tsx 导出为共享常量，表格侧引用，避免两处维护）。

### C5 趋势区块

面积图与 TrendTable 在同一个 OverviewCard 内视觉一体化：图表与表格间加分隔线/间距节奏；TrendTable 保留 `max-h-56` 内滚动与 sticky 表头（功能不动），但容器边框与卡片内边距对齐，消除「框中框」感。

## 主题与可访问性

- 全部新增样式走 Tailwind 语义令牌（`bg-card`、`text-muted-foreground`、`border-border`、`--chart-N`），禁止新硬编码 hex。
- 标题层级调整保持 `aria-labelledby` 链完整；新增的色点/图标必须 `aria-hidden`，颜色不作为唯一信息通道（表行文字仍是主信息）。
- 键盘可达性不退化：切换器、排序按钮、运行/导出按钮的 focus-visible ring 保持。

## 回滚

纯前端样式与 DOM 结构变更，无数据迁移；回滚 = revert 对应 commit。共享组件的增强全部为增量（新 prop/新文件），单独 revert 不影响其他页面。

## 验证

- `npm run check`（icons + typecheck + lint + vitest + build），cwd 为 `residential-monitor/`
- 截图核对：深色 Mocha + 浅色 Latte 两主题，默认窗 1200×800
- `node C:\Users\lyh\.skillsmanage\skills\impeccable\scripts/detect.mjs --json <改动的 tsx/css>` 一次
