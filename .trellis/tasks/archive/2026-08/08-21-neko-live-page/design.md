# 设计：实时连接页

## 1. 组件结构

```
components/features/live/
  index.tsx              页面装配
  hotspot-cards.tsx      两张方向热点卡
  filter-workspace.tsx   快速区 + chip 区 + 编辑器
  filter-chip.tsx        单条已应用条件
  filter-editor.tsx      字段 / 比较方式 / 值 / 单位
  column-menu.tsx        列显隐 popover
  connection-table.tsx   colgroup + thead(排序 + aria-sort) + tbody
  column-resizer.tsx     单个列宽把手（pointer + 键盘）
  empty-state.tsx        六种专门空态
```

## 2. 状态所有权与 IPC 边界

按父任务 `design.md` 第 5 节，`components/**` 不得直接 `invoke`。本页的 IPC 收敛在 `hooks/use-live-page.ts`：

```ts
function useLivePage(input: { applied: ConnectionFilter; sort: LiveSortState; cursor?: ConnectionCursor }): {
  page: ConnectionPage | null;
  loading: boolean;
  errorZh: string | null;
  closeConnection: (identity: string) => Promise<CloseState>;
  saveLayout: (layout: LiveTableLayout) => Promise<void>;
}
```

内部持有 `requestSeq`，四个查询触发源（筛选应用、排序变化、分页 cursor、Channel delta 触发的刷新）共用同一个 seq；失败保留上次结果并单独暴露 `errorZh`。

`features/live/index.tsx` 持有视图状态，不上提到 `app.tsx`：

```ts
type LiveState = {
  applied: ConnectionFilter; // 提交给后端的权威筛选
  draft: FilterDraft; // 编辑中，不触发查询
  sort: LiveSortState; // src/live-table-sort.ts
  layout: LiveTableLayout; // src/live-table-layout.ts
  closeMarks: Map<string, CloseMark>;
};
```

`page` / `loading` / `errorZh` / `requestSeq` 属于 `useLivePage`，不在组件状态里重复一份。

```
```

`applied` / `draft` 的转换、`layout` 的 clamp、`sort` 的翻转全部调用既有纯函数模块，组件里不重写这些规则。

`closeMarks` 沿用现有语义：按 `identity` 记 `accepted` / `closed` / `unconfirmed`，有标记时按钮禁用。

## 3. 列宽拖动

`column-resizer.tsx` 用 `setPointerCapture` + `pointermove` / `pointerup` / `pointercancel` / `lostpointercapture` 四事件，键盘用 `ArrowLeft` / `ArrowRight` / `Home` / `End`。

拖动期间**不走 React state**：直接改 `<col>` 元素的 `style.width` 与表宽（ref 直写），避免每帧重渲整表。`pointerup` 时才 `setState` + 调 `save_live_table_layout` 一次。这与现有 `main.ts:2300-2369` 的做法一致，只是换了宿主。

clamp 与「至少一列可见」由 `src/live-table-layout.ts` 的既有函数负责，组件只传值。

## 4. 表格不按内容重排

`<table className="table-fixed">` + `<colgroup>` 逐列像素宽度。`tbody` 行的 key 用 `row.identity`（epoch:connectionId），保证实时刷新时 React 复用行节点而不重建。

单元格内容作为文本子节点渲染，不用 `dangerouslySetInnerHTML`。

现有 `liveRowCell` 只存在于 `main.ts:236-264`（唯一调用点 `:413`），**不在 `src/format/` 里，也没有测试覆盖**。它返回 HTML 片段字符串，随 `main.ts` 一起废弃。

替代：在 `connection-table.tsx` 里写一个 `cellOf(view, column) -> { text, numeric, title? }`，输入仍是 `displayLiveRow` 的返回值（`src/format/live-row.ts:52` 起，有 `live-row.test.ts` 覆盖，不改）。新函数配自己的单测，覆盖每个 `DataColumnId` 与未知值回退。

所以本子任务**不修改 `src/format/` 的任何文件**。

## 5. 滚动位置

现有整页重绘靠手工存取 `.live-table-wrap` 的 `scrollTop/scrollLeft`（`main.ts:1379-1381,1453-1457`）。React 下滚动容器节点在筛选/排序变化时不被卸载，位置天然保持；但**列显隐变化会改总表宽**，横向位置可能被浏览器夹取。

处理：`column-menu` 提交后不做滚动补偿。本次未打开 Tauri 窗口，未能实拍横向滚动位置。代码把表格滚动容器做成稳定节点（筛选 / 排序不卸载；列显隐只改 colgroup）。待有窗口后按第 17 步实拍，若跳到 0 再按新总宽等比还原。

## 6. 请求竞态

`requestSeq` 单调递增，响应回来时比对，非最新则丢弃。触发查询的四个来源：筛选应用、排序变化、分页 cursor、Monitor Channel 的 `connectionDelta` 触发的刷新。四者共用同一个 seq。

Monitor Channel 的增量归约仍由 `src/ipc/reducer.ts` 负责，产出的是概览快照与健康状态；连接表的行**只来自 `query_live_connections`**，不从 delta 拼行——与现有行为一致。

## 7. 新增 UI 基元

路径：`residential-monitor/src/components/ui/`。风格对齐 shadcn/ui `new-york`（`data-slot`、Radix、cva/cn、无 `animate-in` 依赖）。供家宽页与报告页复用，不要在 `features/**` 内复制同名组件。

| 文件 | 导出 API |
|---|---|
| `input.tsx` | `Input(props: React.ComponentProps<"input">)` |
| `select.tsx` | `Select` `SelectTrigger` `SelectContent` `SelectItem` `SelectValue` `SelectGroup` `SelectLabel` `SelectSeparator` `SelectScrollUpButton` `SelectScrollDownButton` |
| `switch.tsx` | `Switch(props: React.ComponentProps<typeof SwitchPrimitive.Root>)` |
| `popover.tsx` | `Popover` `PopoverTrigger` `PopoverContent` `PopoverAnchor` |
| `table.tsx` | `Table`（额外 `containerClassName?: string`，容器默认 `overflow-auto`）`TableHeader` `TableBody` `TableFooter` `TableHead` `TableRow` `TableCell` `TableCaption` |
| `tabs.tsx` | `Tabs` `TabsList` `TabsTrigger` `TabsContent` |
| `scroll-area.tsx` | `ScrollArea` `ScrollBar` |

实时表不用 `ScrollArea`：用原生 `overflow-auto` 的 `.live-table-wrap`，避免 Radix viewport 改滚动模型。筛选编辑器的字段 / 比较 / 单位用原生 `<select>`，以保留失焦提交时 `contains(activeElement)` 语义。

## 8. 兼容与回滚

- 无 Rust 改动。
- 纯逻辑层只改 `liveRowCell` 的返回形状一处，其余模块只被调用。
- 回滚是把 `app.tsx` 的 live 路由改回 `<PagePending />`。
