# 实时连接固定列宽交互

## Goal

让实时连接表在实时刷新、排序、筛选、隐藏列和拖动列宽时保持稳定：只调整目标列，整表横向滚动可预测，重绘不按内容重新分配宽度。

## Confirmed facts

- 父任务：`08-20-live-connection-ux-optimization`。本子任务只负责 table/colgroup/CSS、resize 状态和 `live_table_layout` 持久化，不负责筛选语义或热点摘要。
- 当前 table/colgroup 在 `residential-monitor/src/main.ts:374-396`，布局 clamp/默认/隐藏/总宽度在 `src/live-table-layout.ts:18-159`，CSS 在 `src/styles.css:463-588`，拖动事件在 `src/main.ts:1417-1463`。
- 既有约束：像素列宽、`table-layout: fixed`、最小/最大宽度、至少一列可见、布局写本机设置键，Recovery 无库时只改内存。

## Requirements

- 显式 table pixel width + `<colgroup>` 作为唯一尺寸源；wrapper 占可用宽度并提供横向滚动，内容长度不能改变列宽。
- resize handle 使用 pointer capture；统一处理 pointerup、pointercancel、lostpointercapture、窗口失焦；一次拖动只持久化一次，保存失败保留内存布局并可诊断。
- 拖动只改变目标数据列；隐藏/恢复、恢复默认、主题/语言切换和整页重绘不缩放其它列；滚动位置保持。
- resize handle 提供 focus/键盘替代或等价可达说明；原生 table、单行省略、排序 `aria-sort` 保持。
- 保留非法布局回退、clamp 和最后一列不可隐藏的安全规则，并补回归测试。

## Acceptance Criteria

- [ ] AC1：连续刷新/排序/筛选后，每个可见列的像素宽度不因内容变化而抖动。
- [ ] AC2：拖动目标列只改变该列，整表宽度与横向滚动一致；取消/失焦不会残留 dragging 或重复保存。
- [ ] AC3：布局重启/主题切换/隐藏列后持久化宽度和显隐正确，非法 payload、clamp、最后一列可见测试通过。
- [ ] AC4：鼠标与键盘/焦点状态可用，相关前端 typecheck/lint/unit tests 通过。

## Out of scope

- 不实现列重排、表头 Excel 菜单、虚拟化、分页、筛选工作区或摘要 DTO。
