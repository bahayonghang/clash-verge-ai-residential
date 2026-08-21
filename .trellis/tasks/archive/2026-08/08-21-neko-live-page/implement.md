# 实施：实时连接页

1. [x] 补齐 UI 基元（若基座未建）：`input`、`select`、`switch`、`popover`、`table`、`tabs`、`scroll-area`，并在 `design.md` 第 7 节登记 API。
2. [x] 在 `connection-table.tsx` 写 `cellOf(view, column) -> { text, numeric, title? }`，输入是 `displayLiveRow` 的返回值。配单测覆盖每个 `DataColumnId` 与未知值回退。**不改 `src/format/live-row.ts` 及其测试**——`liveRowCell` 只存在于 `main.ts:236`，随 `main.ts` 一起废弃。
3. [x] 写 `connection-table.tsx`：`table-fixed` + `colgroup` 像素宽 + `thead` 排序与 `aria-sort` + `tbody` 按 `identity` 作 key + 单行省略。
4. [x] 写 `column-resizer.tsx`：pointer capture 四事件 + 键盘四键；拖动期 ref 直写 `<col>` 宽与表宽，`pointerup` 才 setState 并持久化一次。clamp 走 `live-table-layout.ts`。
5. [x] 写 `column-menu.tsx`：列显隐 popover，保留「至少一列可见」与「恢复默认」。
6. [x] 写 `filter-workspace.tsx` / `filter-chip.tsx` / `filter-editor.tsx`：applied / draft 分离走 `live-filter-workspace.ts`，字段与单位走 `format/live-filter-units.ts`；Enter / 应用 / 失焦 / Escape / 取消五条路径。
7. [x] 写 `hotspot-cards.tsx`：数据只取 `ConnectionPage.summary`；七种 `live.hotspot.status.*` 状态分支。
8. [x] 写 `empty-state.tsx`：`liveEmptyKind` 六态 + 「去设置页」「重新订阅」。
9. [x] 写 `hooks/use-live-page.ts`（IPC 收敛 + `requestSeq`，四个触发源共用一个 seq）与 `index.tsx` 装配。`components/**` 内不得出现 `invoke`。
10. [x] 关闭连接：经 `useLivePage.closeConnection` 调 `close_connection`，按 `CloseState` 三态写 `closeMarks` 并禁用按钮。
11. [x] 补 `zh.ts` / `en.ts` 新键（既有 `live.*` 键沿用不改名）。**不新增键集合一致性测试**——`src/i18n/index.test.ts:7-8` 已有。
12. [x] 替换 live 路由的 `<PagePending />`。**不动 `main.ts`**：整页重绘的焦点/选区/滚动补偿随 `main.ts` 在父任务收口时整体删除，逐页拆除既无收益也会与并行子任务冲突。
13. [x] 跑 `npm --prefix residential-monitor run typecheck && lint && test && build`；`cargo test --workspace` 确认未受影响。

## 实拍检查

14. [ ] 六种空态逐一触发：未配置、未连接、暂停、订阅缺口、已连接无行、筛选无匹配。未打开 Tauri 窗口，单测覆盖文案与分支。
15. [ ] 七种热点状态逐一触发；暂停与缺口下确认不显示数值也不显示 0。未打开 Tauri 窗口，`hotspot-cards.test.tsx` 覆盖七态与隐藏数值。
16. [ ] 列宽拖动：正常松手、窗口失焦取消、`pointercancel`、持久化失败三条路径。未打开 Tauri 窗口。
17. [ ] 切换筛选 / 排序 / 列显隐后的滚动位置行为，把结论写回 `design.md` 第 5 节。结论：未实拍；滚动容器保持挂载。
18. [ ] 四款主题 × 中英文 × 1200×800 / 窄窗口；`aria-sort`、键盘可达、`prefers-contrast: more`。未打开 Tauri 窗口。

## 回滚点

第 12 步之前界面仍是占位，可随时中止。本子任务不改 Rust、不改 `src/format/**`、不改 `src/ipc/**`，回滚只涉及新增的组件与 hook。
