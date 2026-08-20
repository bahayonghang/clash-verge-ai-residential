# 实施：筛选工作区

1. [ ] 抽离 draft/applied 状态与 request token，建立纯函数 apply/clear/remove。
2. [ ] 重写筛选区域 markup/CSS，补应用、取消、清空、命中状态和窄布局。
3. [ ] 调整事件委托，避免 keypress 查询，处理 blur/Enter/Escape/重复提交/过期响应。
4. [ ] 补 TS 单测与中英文案检查；运行 `npm run typecheck && npm run lint && npm test`。

回滚点：保留现有 `liveQuery.filter` 和 `toQueryClause`，若 draft 交互失败可恢复旧布局，不影响后端契约。
