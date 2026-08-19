# 实施计划：实时筛选工具条

## 启动前门禁

- [ ] 父规划已批准。
- [ ] 主题子任务已提供 `.btn-secondary` 与 `label.inline`。
- [ ] 本子任务已 `task.py start`。

## 执行顺序

1. `renderLive` 改为 toolbar + table wrap。
2. CSS：横向开关、次要按钮、表格吃高。
3. 确认现有筛选 / 空态测试仍绿。

## 验证

`npm --prefix residential-monitor run typecheck && npm --prefix residential-monitor run lint && npm --prefix residential-monitor test && npm --prefix residential-monitor run build`

## 回滚

恢复当前 `live-filters` 块。查询层无 diff。
