# 侧栏分色、图表浮层与排名表头可读性

## Goal

操作者在暗色主题下打开主机 / 规则 / 链路 / 进程等聚合页时，能从侧栏分色认出各页、读清条形图悬停数值、并看出排名表哪一列在排序。不改查询、核算、下钻语义。

## Background / Confirmed facts

证据来自用户标注的规则页截图（Image #1，Mocha、近 24 小时、Top 20）与 `refactor/neko-ui-port` 实现。用户已确认未选中导航用 **图标色井**，选中态仍是整行主色蓝。

### 截图三处缺陷

1. **侧栏九段业务导航**：未选中项图标与标题同为 `text-muted-foreground`，项间距 `space-y-1`（4px），九条连成一块灰字。
2. **条形图悬停浮层**：默认 Recharts 浮层写 `IPCIDR` / `value : 4.6 GiB`。`--popover` 在暗色三档与 `--card` 同值（Mocha 均为 `#171c2b`），且未设文字色，对比不足。
3. **排名表头区**：可排序列无可见图标；默认下行降序读不出来。图卡与表各渲染一次「字段归因: 完整; …」，图下那句与 muted 表头叠成一条带。

### 实现锚点

- 导航：`residential-monitor/src/components/layout/sidebar.tsx:172-219`。项高 `--nav-item-py`（comfortable `0.75rem` / compact `0.32rem`，`globals.css:206-216,318-321`）。
- 口径卡色井先例：`caliber-card.tsx:39-48`、`stat-card.tsx:26-35`，背景为色值 15% 透明、图标走该色。
- `RankBar` 默认 `Tooltip`：`charts/rank-bar.tsx:112-120`，`dataKey="value"`，Y 轴 `fill: "#888888"`（`:109`）。同一组件还用于 `drilldown-panel.tsx:112` 与 `residential/aggregate-section.tsx:105`。
- `TrendArea` 已有可读自定义浮层：`trend-area.tsx:45-79`。
- `ShareDonut` 报告页传入 `onHover` 后关闭 Recharts 浮层（`share-donut.tsx:91-96`，`share-donut-card.tsx:29`）。无 `onHover` 时仍走默认浮层。
- 排名表：`dimension/rank-table.tsx`。可排序列名称 / 上行 / 下行 / 连接（`:18,113-132`），默认 `download` 降序（`:52-53`）。`aria-sort` 已有，无图标。`AttributionQualityNote` 在 `rank-bar-card.tsx:84` 与 `rank-table.tsx:107` 各一次。
- 报告 `RankingTable`（`reports/ranking-table.tsx:55-64`）thead 已有 `bg-muted/40`，排序仍无图标。
- 英文 220px compact 契约：归档 `08-21-en-sidebar-layout`。侧栏宽度 160–352、默认 220。Recovery-only 不渲染九段业务导航。
- `DESIGN.md:100-101` One Blue Rule 原句把彩色徽章排除在导航外。本任务把范围改成：选中行与主按钮仍只用主色蓝；未选中业务导航允许色井。

## Requirements

### R1. 侧栏图标色井与标题间距

- 九段业务导航未选中时，左侧为路由色圆角色井 + 同色图标。映射固定，四主题共用语义：概览蓝、实时青、家宽琥珀、主机紫、规则绿、链路青蓝、进程橙、报告靛、告警红。色值走 `--nav-*` token（图表色或 destructive），井底用 `color-mix` 约 18% 透明，禁止把 `#RRGGBB` 拼 `15` 接到 CSS 变量上。
- 选中态仍是整行 `--primary` 圆角条。色井改为白图标 + 浅白井底，不被路由色替换。
- 关于 / 设置保持中性：无色井，未选中 muted，选中设置仍主色整行。
- 未选中 hover：井与图标保持路由色，行底沿用 `hover:bg-secondary`。
- comfortable 用 `--nav-item-gap` 拉开标题（约 0.5rem）；compact 约 0.25rem。不靠加大 `--nav-item-py` 拉开标题。
- 英文 220px compact Medium：品牌两行锁、导航单行、底栏 `Settings / data` 完整；800px 高窗口底栏仍可见。160px 允许 truncate，图标与色井仍在。
- Recovery-only 仍不渲染九段业务导航。

### R2. 暗色图表浮层可读

- `RankBar` 使用自定义浮层（对齐 `TrendTooltip`）：不透明抬升底、可见边、阴影、`text-popover-foreground`。标题为条目名，数值走现有 `valueFormatter`。界面不得出现英文 `value :`。
- 无 `onHover` 的 `ShareDonut` 使用同一浮层外壳。报告页已有 inspect hover 时仍不叠 Recharts 浮层。
- 暗色三档 `--popover` 明显高于 `--card`，边不透明。Latte 保持白底。Radix 浮层跟 token。
- `RankBar` Y 轴刻度改 `currentColor` 或 `var(--muted-foreground)`，删除 `#888888`。

### R3. 排名表头与排序图标

- `RankTable` 与报告 `RankingTable` 可排序列始终显示 lucide 图标：未激活 `ChevronsUpDown`（muted）；激活降序 `ChevronDown`、升序 `ChevronUp`（前景/主色）。
- thead 与表体分层：背景（`bg-muted/40` 或同级）、字重高于表体。`aria-sort` 与图标一致。默认下行降序时「下行」带着降序图标。
- 不可排序列（排名、份额、下钻）无图标、不可点。
- 维度页字段归因只保留在图卡 `RankBarCard`；`RankTable` 不再重复同一句。
- 排序手势、默认列、分页、未知行、下钻开关保持现状。

## Out of scope

- 改 RouteId、查询、核算、下钻能力、时间窗、Top N 语义。
- 改侧栏宽度契约、持久化命令、品牌锁、口号。
- 实时连接表排序标记与列宽拖动。
- 选中导航改成每路由一块彩色底。
- 新视觉世界、远程字体/图标、动画库。
- 首页概览网格、实时筛选条。
- 报告扇形图的 inspect 钉住交互。

## Acceptance Criteria

- [ ] AC1 (R1)：四主题 × 中/英下，九段未选中导航色井颜色互不相同且能扫读；当前页仍是主色整行白图标；关于 / 设置无色井。
- [ ] AC2 (R1)：comfortable 导航标题间距大于现状 `space-y-1`；compact + 220px + 800px 高窗口底栏关于 / 设置仍可见；英文 220px compact 标签单行契约仍成立。
- [ ] AC3 (R2)：Mocha / Macchiato / Frappé 下悬停条形图，浮层底与卡片底可区分，标题与数值可读；中文界面无 `value :`；字节格式与表内一致。
- [ ] AC4 (R2)：`RankBar` Y 轴不再使用 `#888888`；无 inspect 的 `ShareDonut` 走同一浮层外壳。
- [ ] AC5 (R3)：维度排名表进入页面时「下行」带降序图标；点击名称 / 上行 / 连接后图标与 `aria-sort` 同步；排名 / 份额 / 下钻无图标。
- [ ] AC6 (R3)：维度页同一句字段归因只出现一次（图卡内）；表头有独立背景/字重。
- [ ] AC7 (R3)：报告 Top N 表可排序列有可见排序图标。
- [ ] AC8：`sidebar.test.tsx`、`rank-table.test.tsx` 覆盖色井/间距结构、默认降序图标、归因不重复；`npm --prefix residential-monitor` 的 typecheck / lint / test / build 通过。
- [ ] AC9：1200×800 实拍规则页 Mocha 与 Latte：侧栏色井、条形图悬停、表头三处；compact 侧栏底栏仍在。

## Technical Notes

- 包：`residential-monitor` 前端。不改 Rust、DTO、IPC。
- 复杂度：壳 + 主题 token + 两张排名表，且修订 `DESIGN.md` One Blue Rule，需要 `design.md` + `implement.md`。单任务不拆子任务，三处一起看对比。
- 模式：Operate refinement。实施前读 Impeccable `craft-floor`。
- 新可见字符串同时进 `zh.ts` / `en.ts`。禁止远程 URL。
