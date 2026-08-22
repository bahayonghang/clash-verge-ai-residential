# 父任务实施

## 启动前门禁

- [ ] 用户已批准本规划摘要。
- [ ] 不在父任务上 `task.py start`。
- [ ] 先启动并完成 `08-19-ui-locale-zh-en`，再启动 `08-19-live-table-filter`。

## 顺序

1. 语言子任务：设置、目录、托盘、通知、错误、`uiLocale`。
2. 实时表子任务：列、元数据、家宽与自定义筛选。
3. 父任务对照 AC1–AC7 做一次集成核对，再归档子任务与父任务。

## 验证

- `npm --prefix residential-monitor run typecheck`
- `npm --prefix residential-monitor run lint`
- `npm --prefix residential-monitor test`
- `npm --prefix residential-monitor run build`
- 相关 `cargo test`（语言目录、查询筛选、投影速率）
- 不跑 `tinstall`、不写本机 Credential Manager，除非用户另嘱

## 回滚

各子任务自行回滚。父任务无产品代码。
