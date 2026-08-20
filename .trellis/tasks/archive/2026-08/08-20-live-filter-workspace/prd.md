# 实时连接筛选工作区交互

## Goal

把实时连接筛选从混合工具条改成可扫读、可恢复的快速筛选 + 已应用条件工作区，避免文本输入时整页重绘丢焦点，同时保持现有后端过滤语义。

## Confirmed facts

- 父任务：`08-20-live-connection-ux-optimization`。本子任务只负责筛选 UI/会话状态，不修改列宽模型或新增热点摘要 DTO。
- 当前实现位于 `residential-monitor/src/main.ts:311-371,1270-1347`；条件最多 8 条、字段/模式/单位由 `src/format/live-filter-units.ts` 管理，后端语义在 `src-tauri/src/c2/query.rs:74-163`。
- `liveQuery` 的筛选只留当前会话，前端不做分类/Top N/守恒；未知与专门空态继续由现有契约负责。

## Requirements

- 快速区突出 `只看家宽`、已应用条件数量/命中状态、添加条件和清空全部；列显隐入口独立显示。
- 已应用条件以可读 chip/紧凑行呈现，支持单条删除、进入编辑和状态反馈；最多 8 条、AND、空值忽略、字段切换重置模式/单位/值保持不变。
- 编辑使用 draft 状态：每次按键只更新 draft，不触发整页查询；回车/显式应用/失焦提交，Escape/取消恢复。重复提交和过期响应不可覆盖最新结果。
- 提供无匹配/应用中/失败下一步；未配置、断开、暂停、订阅缺口等状态仍使用既有专门空态。
- 中文/英文、键盘、focus-visible、1200×800/窄窗口、prefers-contrast 和 reduced-motion 通过检查；条件文本必须 escape。

## Acceptance Criteria

- [ ] AC1：筛选层级可在桌面与窄窗口扫读，添加/删除/清空/只看家宽均可用键盘操作。
- [ ] AC2：文本编辑期间焦点和光标不因每次 keypress 重绘而丢失；应用/取消/失焦/Escape 行为有测试。
- [ ] AC3：后端收到的查询仍保持已有字段、contains/exact、数值比较、单位转换、8 条 AND 和空值忽略；只接受最新请求响应。
- [ ] AC4：中英文案、专门空态、secret/原始 payload 不泄漏；相关 `npm run typecheck`、lint、unit tests 通过。

## Out of scope

- 不修改后端过滤字段语义，不实现表头过滤菜单、详情抽屉、列宽/列显隐模型或热点摘要。
