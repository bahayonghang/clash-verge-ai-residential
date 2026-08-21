# 实时连接页移植

## Goal

把实时连接页移植到 React 组件系统，视觉换成 neko 的卡片 + 表格形态，功能零回退：后端筛选语义、筛选工作区草稿交互、列宽与列显隐持久化、表头排序、关闭单条连接、方向热点摘要、六种专门空态全部保留。

## 父任务

`.trellis/tasks/08-21-neko-ui-refactor`。目录约定、主题映射、数据流约束见父任务 `design.md` 第 1 / 3 / 4 节。

## 依赖

前端基座（`08-21-neko-shell-foundation`）。本子任务不依赖 C3 能力子任务。

按父任务 `implement.md` 的建立方表，表格与筛选相关的 UI 基元（`input`、`select`、`switch`、`popover`、`table`、`tabs`、`scroll-area`）由本子任务建立并在 `design.md` 第 7 节登记，供家宽页与报告页复用。

按父任务 `design.md` 第 5 节，本页的 IPC 必须收敛在 `hooks/use-live-page.ts`，`components/**` 不得直接 `invoke`。

## Confirmed facts

### 现有实现分布

- 页面渲染 `residential-monitor/src/main.ts:371-600`（`renderLive`）与 `:601-633`（`renderLiveHotspotCard`）。
- 事件处理散在 `main.ts` 的委托里，包含 `live-residential` 复选框（`:2177-2178`）、列宽拖动、排序、关闭连接。
- 纯逻辑层已抽出且有单测，**本子任务只调用，不重写**：
  - `src/live-table-layout.ts` + `.test.ts`：列宽 clamp、可见列、隐藏最后一列的保护、非法布局回退默认模板。
  - `src/live-table-sort.ts` + `.test.ts`：排序状态。
  - `src/live-filter-workspace.ts` + `.test.ts`：applied / draft 分离。
  - `src/format/live-filter-units.ts` + `.test.ts`：字段、比较模式、单位换算。
  - `src/format/live-row.ts` + `.test.ts`：`displayLiveRow` 行展示。
  - `src/format/live-hotspot.ts` + `.test.ts`：热点摘要展示。
  - `src/ipc/live-session.ts` + `.test.ts`：`query_live_connections` 入参构造。
  - `src/ipc/live-empty.ts` + `.test.ts`：`liveEmptyKind` 六种空态判定与文案。
- 列渲染入口 `main.ts:236-264` 的 `liveRowCell(view, column)` 把 `displayLiveRow` 的结果映射到列，`isNumericColumn` 决定右对齐。

### 后端契约

- `query_live_connections` 的入参 `ConnectionQuery`（`residential-monitor/src-tauri/src/c2/query.rs:37-45`）：`filter`、`sort_field`、`descending`、`cursor`、`limit`。默认 `limit = LIST_PAGE_DEFAULT`，上限 `LIST_PAGE_MAX`（`:87-93` 的 `sanitize_limit`）。
- `ConnectionFilter`（`:7-20`）：`category`、`host`、`process`、`rule`、`chain`、`network`、`residential_only`、`clauses`。条件最多取前 8 条（`:106-113` 的 `.take(8)`）。
- 返回 `ConnectionPage`（`:59-67`）：`rows`、`nextCursor`、`matchedCount`、`sampleUtc`、`summary`（`top_download` / `top_upload` 两个 `ConnectionHotspot`）。热点由后端算，前端不从可见行推算。
- 连接行 DTO `LiveConnectionView`（`src/dto.ts:41-65`）。
- 关闭连接 `close_connection` 返回 `CloseState`（`src/dto.ts:255-259`）三态：`accepted` / `closed` / `unconfirmed`。三态文案在 `main.ts:401-408`。
- 列布局持久化 `save_live_table_layout`，取值形状 `{ widths: Record<string, number>; hidden: string[] }`（`src/dto.ts:238`）。

### 整页重绘的手工补偿将被移除

`main.ts:1369-1381` 与 `:1435-1457` 为了整页 `innerHTML` 替换而手工保存/恢复 `activeElement.id`、输入选区与 `.live-table-wrap` 的 `scrollTop/scrollLeft`。这套补偿随 `main.ts` 在父任务收口时整体删除，**本子任务不逐页拆除**（逐页拆除既无收益，也会与并行子任务在同一文件上冲突）。

但**滚动位置在切换筛选 / 排序 / 列显隐时的保持行为要作为验收项显式实测**，不能假设 React 自动等价。

### 目标视觉

- 表格与分页参考 `ref/neko-master/apps/web/components/features/stats/table/{domain-table,ip-table,domain-stats-table,ip-stats-table}.tsx`。
- 卡片壳与标题参考 `ref/neko-master/apps/web/components/common/overview-card.tsx:22-43`。
- 热点摘要用 neko 统计卡形态（`ref/neko-master/apps/web/components/features/stats/stats-cards.tsx:68-108`）。
- Tab 切换参考 `ref/neko-master/apps/web/app/[locale]/dashboard/components/content/index.tsx:117-140`。

## Requirements

### R1. 筛选工作区

- 保留三层结构：快速区（`只看家宽` 主开关、已应用条件数、命中状态、添加条件、清空全部）+ 已应用条件 chip 区（单条删除、进入编辑）+ 条件编辑器（字段 / 比较方式 / 值 / 单位）。
- 保留 draft / applied 分离：每次按键只改 draft，不触发查询；Enter / 显式应用 / 失焦提交，Escape / 取消恢复。
- 保留请求序号与过期响应丢弃：重复提交与迟到响应不得覆盖最新结果。
- 保留后端语义：字段集合、`contains` / `exact`、数值比较、单位换算、最多 8 条 AND、空值忽略、字段切换重置比较模式 / 单位 / 值。
- 列显隐入口与筛选入口在信息层级上分离。

### R2. 表格

- 保留像素列宽 + `table-layout: fixed` 的专家表格模型：实时刷新、排序、筛选、重绘不得按内容重排列宽。
- 保留列宽拖动：只改目标列宽，总表宽随可见列宽之和更新；pointer capture / 窗口失焦的取消路径；松手只提交一次持久化，失败保持内存布局并给可诊断反馈。
- 保留列宽 clamp、至少一列可见、非法布局回退默认模板的安全约束及其回归测试。
- 保留表头排序与 `aria-sort`；保留列调整把手的键盘替代或等价可达说明。
- 保留 `<table>` 语义与单行省略。

### R3. 热点摘要

- 两张卡：最高下载连接、最高上传连接。数据取 `ConnectionPage.summary`，不在前端从可见行推算。
- 保留状态区分：当前快照 / 没有匹配连接 / 采集已暂停 / 订阅存在缺口 / 尚未配置控制器 / 控制器未连接 / 能力或采样未知（`live.hotspot.status.*` 七个键）。暂停与缺口时隐藏数值，不显示旧值也不显示 0。

### R4. 空态与操作

- 保留 `liveEmptyKind` 的六种专门空态与各自的中文下一步，不合并成「无数据」。
- 保留「去设置页」「重新订阅」两个恢复动作。
- 保留关闭单条连接与三态反馈；不新增关闭全部入口。

### R5. 双语与可访问性

- 新增字符串同时进 `zh.ts` 与 `en.ts`；`live.*` 既有键沿用，不重命名。
- 条件值与主机名必须转义（React 默认转义，但 `dangerouslySetInnerHTML` 一律禁止）。
- 保留 `:focus-visible`、`prefers-contrast: more`、`prefers-reduced-motion`。

## Out of scope

- 不改 `ConnectionFilter` / `ConnectionQuery` / `ConnectionPage` 的后端契约。
- 不重写也不修改 `src/live-table-*.ts`、`src/live-filter-workspace.ts`、`src/format/live-*.ts`、`src/ipc/live-*.ts` 及其单测。
- 不改 `src/main.ts`。
- 不实现详情抽屉、虚拟化、列重排、表头过滤菜单、关闭全部连接。
- 不动概览页、聚合页、家宽页、报告页、告警页、设置页。
- 不改 C2 采集生命周期、Monitor Channel、托盘与凭据边界。

## Acceptance Criteria

- [ ] AC1 (R1)：筛选工作区三层结构在四款主题 × 中英文下可扫读；添加 / 删除 / 清空 / 只看家宽 / 列显隐均可键盘操作。
- [ ] AC2 (R1)：文本条件编辑期间焦点与光标不丢失；应用 / 取消 / 失焦 / Escape / 重复提交 / 过期响应六条路径有测试。
- [ ] AC3 (R1)：后端收到的查询仍是既有字段、`contains` / `exact`、数值比较、单位换算、8 条 AND、空值忽略语义（有测试比对入参）。
- [ ] AC4 (R2)：实时刷新、排序、筛选、重绘过程中每个可见列保持持久像素宽度；拖动只改目标列；取消 / 失败路径不遗留 dragging 状态；`live-table-layout` 的既有回归测试全部通过且未删断言。
- [ ] AC5 (R2)：切换筛选与排序后表格横向/纵向滚动位置的行为经实测确认（不依赖对 React 的假设）。
- [ ] AC6 (R3)：两张热点卡的数据来自 `ConnectionPage.summary`；七种状态逐一实测，暂停与缺口下不显示数值也不显示 0。
- [ ] AC7 (R4)：六种专门空态逐一实测，各自的中文下一步正确；「去设置页」「重新订阅」可用。
- [ ] AC8 (R4)：关闭单条连接的三态反馈正确；无关闭全部入口。
- [ ] AC9 (R5)：源码中无 `dangerouslySetInnerHTML`；`zh.ts` / `en.ts` 键集合一致。
- [ ] AC10 (R2)：`components/**` 内无 `invoke` 调用，本页 IPC 全部经 `hooks/use-live-page.ts`（源码级确认）。
- [ ] AC11：`npm --prefix residential-monitor run typecheck && lint && test && build` 通过；本页无 Rust 改动，`cargo test --workspace` 仍通过。
- [ ] AC12 (R2/R5)：1200×800 与窄窗口实拍无横向溢出（表格自身的横向滚动区除外）；`aria-sort` 正确。
