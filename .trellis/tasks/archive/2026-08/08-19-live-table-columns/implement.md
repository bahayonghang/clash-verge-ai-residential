# 实施计划：列宽与显隐

## 启动前门禁

- [x] 父规划已批准。
- [x] `08-19-live-table-columns` 已 `task.py start`。
- [x] 已读 frontend / backend checklist 与父 `design.md`。

## 执行顺序

1. Rust：`LiveTableLayout` sanitize + `put_setting("live_table_layout")` + bootstrap 字段 + 命令。单测：非法 JSON、藏光数据列、action 混入、超长。
2. 前端类型与默认模板常量（与 Rust 默认一致）。
3. CSS：去掉 `.live-table { display: block }`，`table-layout: fixed`，nowrap/ellipsis，手柄。
4. `renderLive`：`colgroup`、显隐、列面板、`data-col`。
5. `paint` 保存/恢复滚动；拖动锁；`pointerup` 保存布局。
6. i18n：列按钮、恢复默认、列名（可复用 `live.col.*`）。

## 验证

```text
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
```

Rust：layout sanitize 与 `save_live_table_layout` 回落。不跑 `tinstall`。

## 回滚

删除设置键 `live_table_layout`；恢复 `renderLive` / CSS / bootstrap 字段。
