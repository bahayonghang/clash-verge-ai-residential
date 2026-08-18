# DTO 解码

- Rust 是权威校验者。前端解码失败时显示专门中文状态，不猜测缺字段。
- 每条 Channel 消息必须检查 `schemaVersion`、`kind` 和单调 `seq`。
- 禁止把 mihomo 原始 JSON 或 SQL 行传到视图层。
- 时间展示用用户本地时区；持久时间保持 UTC integer。

## Scenario: C2 Monitor Channel

### 1. Scope / Trigger
- Trigger: 前端订阅实时状态、窗口重建、序号缺口或 schema 不兼容。

### 2. Signatures
- `subscribe_monitor(on_event) -> subscriptionId`
- `resync_monitor(subscriptionId, on_event) -> newSubscriptionId`
- 首帧：`bootstrap { schemaVersion, subscriptionId, snapshot, baseSeq, backendTime }`
- 后续：`connectionDelta | healthChanged | summaryChanged`，均带 `seq`

### 3. Contracts
- `schemaVersion` 必须为 `1`。
- 后续消息只接受 `seq > baseSeq`。
- `snapshot` 是概览 DTO，不含 10k 连接数组。
- 列表与详情走 `query_live_connections` / `get_connection`。

### 4. Validation & Error Matrix
- `seq == lastSeq + 1` → 应用增量
- `seq <= lastSeq` → 忽略重复或陈旧消息
- `seq > lastSeq + 1` → 冻结并 `resync`
- 未知 `kind` 或错误 `schemaVersion` → fail closed，显示升级 / 重载
- `subscriptionId` 不匹配 → 丢弃迟到消息

### 5. Good/Base/Bad Cases
- Good: bootstrap 后连续 `seq`
- Base: 窗口重建生成新 `subscriptionId`，清空 cache
- Bad: 缺口后继续猜状态；把 gap 显示成 `0`

### 6. Tests Required
- TS `reducer.test.ts`：迟到订阅、缺口、重复、`204` 后等 remove
- Rust `channel_contract_tests`：首帧 bootstrap、resync 换 identity

### 7. Wrong vs Correct
#### Wrong
前端把 `meterUpload` 与 `attributedUpload` 加成「全局流量」，缺口当 0。
#### Correct
分字段展示；`null` 显示「未知」。

