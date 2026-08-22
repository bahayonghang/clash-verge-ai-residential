# 实施计划：概览成对口径

## 启动前门禁

- [ ] 父规划已批准。
- [ ] `08-19-ui-catppuccin-theme` 已合并或至少语义 token 可用。
- [ ] 本子任务已 `task.py start`。

## 执行顺序

1. 分类并集纯函数 + 测试。
2. `renderOverview` 换成口径组 + 状态区 + 分类表。
3. `.caliber-grid` 与空态 CSS。
4. 中英文键。

## 验证

`npm --prefix residential-monitor run typecheck && npm --prefix residential-monitor run lint && npm --prefix residential-monitor test && npm --prefix residential-monitor run build`

## 回滚

恢复 8 卡 `metric()` 网格。
