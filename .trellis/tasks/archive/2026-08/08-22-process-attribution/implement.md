# 实施：进程归因（父任务）

## 顺序

1. 评审本目录与两个子任务的 `prd.md` / `design.md` / `implement.md`。
2. 用户批准本规划摘要后，`task.py start 08-22-process-lookup-observation`，按该子任务 implement 做完并 `trellis-check`。
3. `task.py start 08-22-process-page-capability`，同样做完并检查。
4. 父任务只做对照：AC1–AC5 均有子任务证据；`just ci` 与 `just monitor-check` 各跑一遍。
5. 不要在父任务目录里改产品代码。

## 验证

- `just ci`
- `just monitor-check`
- `npm run check:secrets`

## 开工前

- 规划摘要已展示，且用户在后续消息中批准实施。
- 未批准前不运行 `task.py start`。
