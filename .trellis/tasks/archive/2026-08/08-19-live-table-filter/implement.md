# 实施计划：实时表列与筛选

## 启动前门禁

- [ ] 用户已批准父任务规划摘要。
- [ ] `08-19-ui-locale-zh-en` 已完成或至少已提供 `uiLocale` 读取。
- [ ] 已读 frontend / backend spec checklist。
- [ ] 不改核算公式、不传 mihomo 原始 JSON、不增加关闭全部。

## 执行顺序

### 1. 归一化与投影

- `normalize_connection` 读取 port / type / start。
- `project_live` 填 `duration_ms`；hub 或投影填 `rate_*`。
- 解码追加可选字段。

**Gate**：无 start → 时间未知；第二帧才有速率；差 0 → `Some(0)`。

### 2. 查询

- `ConnectionFilter.residentialOnly` + `clauses`。
- facade 查询时注入当前 targets。
- 单测：默认目标「家宽」命中 `AI-家宽`；精确不命中子串；AND；旧查询缺省不过滤家宽。

### 3. 表格与筛选 UI

- 十二列 + 操作。表头跟 `uiLocale`。
- 「只看家宽」默认开。条件行最多 8。
- `queryLiveConnections` 传当前视图筛选。

**Gate**：TS 测试覆盖展示纯函数与查询参数；`live-session` 不再永远空筛选。

## 验证

- `npm --prefix residential-monitor run typecheck && npm --prefix residential-monitor run lint && npm --prefix residential-monitor test && npm --prefix residential-monitor run build`
- `cargo test` 覆盖 `c2::query` 与归一化 / 速率
- 不跑 `tinstall`

## 回滚

恢复七列与空筛选；撤回 Meta / Filter 追加字段。
