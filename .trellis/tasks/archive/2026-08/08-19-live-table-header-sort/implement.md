# 实施计划：表头排序

## 启动前门禁

- [ ] 父规划已批准。
- [ ] `08-19-live-table-columns` 已完成或至少已有 `data-col` 表头。
- [ ] 本子任务已 `task.py start`。

## 执行顺序

1. `sort_key` + 未知在后比较 + 游标。Rust 单测覆盖速率/时长/链路/类型与 identity 平局。
2. `live-session` 仍默认 identity；允许调用方改 `sortField`。
3. `renderLive` 表头可点；`aria-sort`；i18n 方向提示。
4. 点击更新 `liveQuery`、清 cursor、刷新。delta 路径使用当前 `liveQuery`。
5. 更新测试：`live-session.test.ts` 默认 identity；新增排序循环纯函数若抽出。

## 验证

前端四门 + `cargo test` 覆盖 `c2::query` 排序。不跑 `tinstall`。

## 回滚

`sort_key` 回落 identity；表头恢复纯文本。
