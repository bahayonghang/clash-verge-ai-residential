# 设计：neko 界面移植总体结构

本文件只定义跨子任务的边界、契约与目录约定。单页内部结构写在各子任务的 `design.md`。

## 1. 目标目录结构

```
residential-monitor/
  index.html                    挂载点改为 React 根，保留 skip link
  postcss.config.mjs            新增，@tailwindcss/postcss
  src/
    main.tsx                    createRoot + 引导（原 main.ts 的 bootstrap 逻辑）
    app.tsx                     应用壳：路由状态、Monitor Channel 订阅、偏好加载
    styles/globals.css          Tailwind 入口 + neko 令牌 + 四款主题变体
    components/
      ui/                       Radix + cva 基元：card button badge tabs tooltip
                                dropdown-menu select switch input table separator
                                scroll-area popover skeleton
      common/                   stat-card overview-card top-list-item
                                time-range-picker theme-toggle language-switcher
                                status-dot
      layout/                   sidebar header shell page-pending
      features/
        overview/               口径卡组 + 趋势图 + Top 三列
        dimension/              主机/规则/链路/进程 共用的聚合页骨架
        live/                   连接表 + 筛选工作区 + 热点卡
        residential/            家宽三段
        reports/                分析报告
        alerts/                 告警
        settings/               设置 / 数据管理
        recovery/               recovery 壳 + 不可用页
      charts/                   Recharts 封装：trend-area rank-bar share-donut
    hooks/                      use-bootstrap use-monitor-stream use-preferences
                                use-sidebar-resize use-report use-residential-share
                                use-live-page use-alerts use-settings
    lib/                        cn() 与格式化再导出
    ipc/                        保留：decoder live-session reducer secret-field
    format/                     保留：units live-row live-hotspot report-view
                                report-inspect overview 等纯函数
    i18n/                       保留 zh.ts / en.ts，新增键
    theme.ts                    保留解析函数，applyTheme 语义不变
    dto.ts                      保留，按需扩展
```

Tailwind v4 不需要 `tailwind.config`：`@theme` 写在 `styles/globals.css` 里。

`src/main.ts` 与 `src/styles.css` 在全部子任务完成后删除。

## 2. 稳定层与可删层

**稳定层，只调用不重写**：`src/ipc/**`（decoder / reducer / live-session / live-empty / secret-field）、`src/live-table-layout.ts`、`src/live-table-sort.ts`、`src/live-filter-workspace.ts`、`src/shell-width.ts`、`src/theme.ts`，以及 `src/format/` 里的**数据与解码纯函数**：`units.ts`、`overview.ts`、`live-row.ts`、`live-hotspot.ts`、`live-filter-units.ts`、`report-view.ts`，`report-inspect.ts` 的 key/model 纯函数。

**可删层，明确批准删除**：
- `src/format/report-svg.ts` 与 `report-svg.test.ts` —— 手写 SVG 渲染，被 Recharts 封装取代。归 `08-21-reports-alerts-settings-port`。
- `src/format/report-inspect.ts` 里依赖 DOM 与整页重绘的四个函数（`readReportScroll` / `writeReportScroll` / `inspectKeyExists` / `shouldSkipRoutinePaint`）及其用例。同上归属。
- `src/main.ts` 里为整页 `innerHTML` 替换做的焦点 / 选区 / 滚动补偿（`:1369-1381`、`:1435-1457`）—— 随 `main.ts` 整体删除，不逐页拆除。

区分标准：**渲染实现可删，数据与解码逻辑不可删**。

## 3. 保留不变的契约

- 59 个 Tauri command 的名称、参数与返回结构不变（`generate_handler!` 在 `residential-monitor/src-tauri/src/lib.rs:1237-1296`，registered 与 declared 均为 59）。新增命令只在 `08-21-c3-dimension-capability` 与 `08-21-residential-page` 内，且只增不改。
- `MonitorStreamMessage` 五种 kind 与 `src/ipc/reducer.ts` 的归约语义不变。
- `UiTheme` 四个值、`UiFont`、`UiFontSize`、`UiDensity`、侧栏宽度的持久化命令与取值范围不变。
- `liveTableLayout` 的宽度 clamp、至少保留一列、非法布局回退默认模板不变（`src/live-table-layout.ts`）。
- `decodeAbout` 拒绝 `signed: true` 的断言不变（`src/dto.ts:379-387`）。
- `Granularity` 的既有 `Hour | Day | Month` kebab-case 序列化值不变（新增分钟档只增不改）。

## 4. 主题映射

Tailwind v4 的 `@custom-variant` 无法覆盖四值枚举，改用属性选择器分四段声明令牌：

```css
:root[data-theme="latte"]      { /* neko light 令牌 */ }
:root[data-theme="frappe"]     { /* neko dark，背景抬亮一档 */ }
:root[data-theme="macchiato"]  { /* neko dark，中间档 */ }
:root[data-theme="mocha"]      { /* neko dark 原值 */ }
```

`applyTheme` 继续只写 `data-theme` 与 `color-scheme`，light/dark 判定沿用 `theme.ts:79` 的 `theme === "latte"`。深色三档只调 `--background` / `--card` / `--sidebar` 的明度，`--primary` 与 `--chart-1..5` 三档共用，避免出现三套图表配色。

`prefers-contrast: more` 与 `prefers-reduced-motion` 在 `globals.css` 里统一覆盖：前者提高 `--border` 与 `--muted-foreground` 对比，后者关闭数字过渡、shimmer、ping 与 `animate-progress-indeterminate`。

## 5. 数据流

```
Rust command / Monitor Channel
        │
        ├── src/ipc/decoder.ts  边界解码（保留）
        │
        ├── src/ipc/reducer.ts  流式归约（保留）
        │
        └── hooks/use-*.ts      React 状态：只存视图选择与 DTO 缓存
                    │
                    └── components/**  纯展示 + 事件回调
```

约束：`components/**` 不得直接 `invoke`，全部 IPC 收敛在 `hooks/**`。这条对每个页面子任务都成立，因此每页至少有一个 hook：

| 页面 | hook | 归属子任务 |
|---|---|---|
| 应用壳 | `use-bootstrap` / `use-monitor-stream` / `use-preferences` / `use-sidebar-resize` | 基座 |
| 概览 + 四聚合页 | `use-report` | 概览聚合 |
| 实时连接 | `use-live-page` | 实时页 |
| 家宽 | `use-residential-share`（+ 复用 `use-report`、`use-live-page` 的查询） | 家宽页 |
| 报告 / 告警 / 设置 | `use-report`（复用）/ `use-alerts` / `use-settings` | 报告告警设置 |

每个 hook 必须实现：请求序号递增与过期响应丢弃、失败保留上次结果并单独暴露 `errorZh`。不做静默失败。

聚合页的排名与合计一律来自 Rust 查询；组件只做排序展示与百分比渲染，百分比分母取 `totals`，不取可见行之和（`PRODUCT.md:38` 的既有约束，本次不改）。

## 6. 图表封装边界

`components/charts/` 暴露三个封装，各页共用：

- `trend-area`：双序列面积图。输入 `{ bucketUtc, upload, download }[]`，颜色固定 `#3b82f6` / `#a855f7`，`isAnimationActive={false}`。建立方：概览聚合子任务。
- `rank-bar`：横向条形图。输入 `{ label, value }[]` + 色序 `--chart-1..5`。建立方：概览聚合子任务。
- `share-donut`：占比环图。输入 `{ label, value }[]`。建立方：报告页子任务。

每个封装必须接受 `emptyHint` 与 `loading`，无数据时渲染虚线边框空态（对照 `ref/neko-master/apps/web/components/features/stats/charts/trend-chart.tsx:235-243`），不渲染 0 高度的图。每个图表挂载点旁必须有对应数据表或 Top 列表。

## 7. Rust 改动的归属

**归 `08-21-c3-dimension-capability`（Rust only 子任务）**：分钟粒度、规则/链路派生聚合键、`ReportFilters` 注入到全部排名与序列路径、`TOTALS_RAW` 加 category、`dimension_filter_count` 计入 category、Category 排名路由修正、五维物化与 category 维度层排名、能力报告在无物化维度时不谎报 `exact_top_n`、未知维度值不被 INNER JOIN 丢弃。

**归 `08-21-residential-page`**：家宽判定收敛到一个模块（两个具名函数）、`share_residential_raw` named SQL、`residential_share` 命令。**不新增表，不改 schema，不改通用 C3 查询。**

**归 `08-21-neko-shell-foundation`**：`list_routes` 的路由列表扩展（纯增量）。

**其余子任务不得改 Rust。** 概览聚合与实时页、报告告警设置三个子任务是纯前端。

## 8. 从 neko 借用的契约，不借用的实现

**借用**：`buildRuleName` 的归并语义（多跳归最后一跳，`ref/neko-master/apps/collector/src/shared/utils/rule-name.ts:7-29`）；分钟层 + 小时层 + 日层的分层思路；跨维下钻的表达形式（规则 → 链路 / 域名，域名 → IP）。

**不借用**：neko 按维度独立建表的 schema（`domain_stats` / `ip_stats` / `proxy_stats` / `rule_stats` / `device_stats` 等十余张）。

这个决定的依据必须写清：本项目的 `dimension_dict` + `traffic_hourly_dimension` / `traffic_daily_dimension` / `traffic_daily_core` 是一套**通用**分层，`dimension_kind` 列可承载任意维度。它当前的问题不是表达力不足，而是**只物化了 `'host'` 一种 kind**（`c3/retention.rs:103-119` 写死字符串 `'host'`）。把物化扩到五个维度即可，不需要换表结构。这条修复归 `08-21-c3-dimension-capability`。

**不借用**：neko 的 GeoIP 服务（MaxMind MMDB + 远程兜底）、ClickHouse 读写分层、agent 探针协议、Surge 适配、多后端隔离（`backendId`）。

## 9. 跨维下钻与能力的诚实降级

`ReportResult.drilldownCapability` 已是现成契约（`c3/query.rs:615-681`）：

| 查询区间 | tier | cross_dimension | sessions | exact_top_n |
|---|---|---|---|---|
| 30 天 raw 期内 | `Raw` | true | true | true |
| 超出 raw 期、13 个月内 | `HourlyDimension` / `DailyDimension` | false | false | true |
| 更久 | `DailyCore` | false | false | false |

**第二行的 `exact_top_n: true` 目前是谎报**：只有 host 有物化，其余维度返回空排名。修正后该 flag 反映 grouping 是否真有物化数据（归 C3 子任务）。

前端一律由这三个 flag 驱动：`cross_dimension: false` 时隐藏下钻并显示 `note_zh`；`exact_top_n: false` 时排行区显示能力说明而不是空表。前端不猜、不缓存旧能力值。

## 10. 取舍

- **选 Recharts 而非手写 SVG**：趋势图的 tooltip、坐标轴刻度、响应式容器与空态是 neko 视觉的主要成本项，手写会持续消耗预算且难以对齐。代价是打包体积增加，桌面本地加载可接受。
- **不引入 React Query**：数据源是本机 IPC 而非 HTTP，没有缓存失效与重试的复杂度；自建 hooks 更小且不多一层心智。
- **不引入 `@xyflow/react`**：链路 DAG 是 neko 里最重的一个依赖，本项目的链路是短链且已有链路字符串，条形图与表格足够表达占比。
- **保留四款主题值而非改成 light/dark**：避免 Rust 契约变更与已持久化值的迁移。
- **把 Rust 查询与物化能力独立成一个子任务**：审阅暴露出这部分的工作量与风险都不属于「界面移植」，混在页面子任务里会让两个前端子任务互相等待同一处 SQL 改动。独立后前端子任务全部是纯前端，依赖关系单向。
- **家宽保留两种判定语义**：合并到精确匹配会改变已发布的「只看家宽」行为；合并到含启发式会把中文子串写进持久化分类键并改变历史分类。收敛实现位置解决漂移，不改行为。

## 11. 中间态与回滚

**中间态**：基座合入后到四个页面子任务完成前，应用不可用于日常使用。业务页渲染 `<PagePending />` 占位。

不做「按页降级到 `main.ts` 旧渲染函数」：旧渲染依赖 `main.ts` 内约 20 个模块级可变状态（`liveFilterDraft`、`reportInspectPinned`、`liveTableLayout` 等），暴露给 React 壳会造成两套状态源同时可写，比占位的过渡期成本更高。

**因此整个重构在 `refactor/neko-ui-port` 分支上完成后一次性合入 `main`**，不分批合入。

**回滚形状**：

| 层级 | 回滚动作 |
|---|---|
| 基座 | `index.html` 的 script 改回 `/src/main.ts`，恢复 `styles.css` 的 link |
| 单个页面 | `app.tsx` 的该路由改回 `<PagePending />` |
| C3 分钟粒度 | 移除新枚举值与 match 分支 |
| C3 派生键 | 还原四处 SQL 与移除标量函数注册 |
| C3 五维物化 | 移除物化分支与 `hourly_dim_v2` 水位行；已写入的新 kind 行留在表里不影响旧读路径 |
| 家宽判定收敛 | 把两个判定还原到原位（行为不变，回滚无用户可见影响） |
| 家宽 named SQL 与命令 | 纯增量，直接移除 |
| 报告页删除的 SVG 与用例 | 需从 git 还原文件 |

## 12. 开放项

- `frappe` / `macchiato` 两档深色的具体明度值需要实拍确认，先按 `--background` 提亮 6% / 3% 起步。
- 五维物化对 30 天库体积与物化耗时的影响：`08-21-c3-dimension-capability` 的 `design.md` 第 5 节，必须实测。
- 分钟档的可选 bucket 与 `connection_minute` 可查询窗口的对齐：同上子任务。
