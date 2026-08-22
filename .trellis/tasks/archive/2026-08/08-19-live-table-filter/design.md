# 技术设计：实时表列与筛选

## 边界

```text
GET /connections
        │ normalize_connection
        ▼
ConnectionMeta + start / ports / inbound
        │ project_live + 上一帧字节
        ▼
LiveConnectionView（duration / rate 可空）
        │ query_connections(residentialOnly, clauses)
        ▼
renderLive（十二列 + 操作，文案跟 uiLocale）
```

不改 C1 核算与 Channel `schemaVersion`。列表仍以 `query_live_connections` 页为准。

## 元数据

`ConnectionMeta` 增加：`source_port`、`destination_port`、`inbound`（控制器 `metadata.type`，如 `Tun` / `Inner`）、`start`（ISO-8601 原文，截断到 `STRING_LIMIT`）。

`LiveConnectionView` 增加同样可选字段（camelCase）。前端解码缺字段当 `null`。

`project_live`：

- `duration_ms`：能解析 `start` 则用 `received_utc * 1000 - start_ms`，失败则 `None`。
- `rate_*`：用上一帧同 `connection_id` 的 upload/download 与时间差。没有上一帧 → `None`。有上一帧 → `Some(delta * 1000 / dt_ms)`，含 0。

上一帧缓存在投影/hub 侧，不写 SQLite，不断开时清空。暂停保留行时也保留速率缓存。

## 展示

纯函数（建议 `src/format/live-row.ts`）：

- 主机：`host` + 可选 `destinationPort`
- 来源/目标：ip + 可选 port
- 规则：`rule` 或 `rule(payload)`
- 链路：`chains.join(" / ")`；空则未知
- 类型：`inbound` 与 `network` 都有则 `Tun(tcp)`，否则能显示哪个显示哪个
- 时间：相对时间，跟 `uiLocale`（中文「3 分钟前」，英文 `3 minutes ago`）
- 未知文案跟 `uiLocale`

Clash 的链路展示顺序本任务保持控制器原序，不反转。

## 查询合同

扩展 `ConnectionFilter`（追加字段，旧字段保留）：

```text
residentialOnly: bool          // 缺省 false（兼容旧调用）
clauses: [{ field, mode, value }]
```

`field`：`host` | `chain` | `rule` | `process` | `source` | `destination` | `type`  
`mode`：`exact` | `contains`  
空 `value` 忽略该行。最多 8 条。全部 AND，再与 `residentialOnly` AND。

家宽：

```text
any(chain == target) OR any(chain contains "家宽")
```

`targets` 来自当前 `AccountingEngine`，查询时由 facade 注入，不让前端传目标列表。

匹配对象：

- host：原始 host，以及 `host:port`（若有端口）
- source / destination：ip，以及 `ip:port`
- type：`inbound`、`network`、以及组合 `Tun(tcp)`
- chain：任一节点；contains 对节点子串
- rule：`rule` 与 `rule(payload)`
- process：`processName`

匹配区分大小写。

前端 `queryLiveConnections(query)` 不再写死空筛选。视图状态保存：`residentialOnly`（默认 true）+ `clauses`。只留当前会话。

旧的单字段 `host`/`chain`/… 仍按包含实现，本任务 UI 不再使用它们。

## 前端

`renderLive` 表头与筛选标签走语言表。筛选区：「只看家宽」开关 + 添加条件 + 每行字段/模式/值 + 删除。变更后重新 `query_live_connections` 第一页。

空态五类不变。筛选后零行用已连接无行文案，可附带「当前筛选无匹配」。

## 风险

- 追加 DTO 字段必须让旧解码测试仍过（可选）。
- 速率缓存不得在 HTTP 持锁期间变大无界：只保留当前活跃 `connection_id`。
- 10k 行筛选仍走现有内存过滤；不在本任务做虚拟化。
