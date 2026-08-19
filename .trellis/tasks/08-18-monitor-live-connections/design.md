# 技术设计：接通实时连接数据通路

## 目标

补齐三层断点，让「实时连接」显示当前投影行，并随 1 Hz 采样更新。不改 C1 核算、Channel 消息形状或 C2 查询合同。

## 边界

```text
HTTP GET /connections
        │ 不持 AppFacade 锁
        ▼
ControllerSession 归一化
        ▼
AppFacade.ingest_snapshot / apply_lifecycle
        │ 返回 Option<MonitorStreamMessage>
        ▼
SubscriptionRegistry 转发已保存的 Tauri Channel
        ▼
decoder + reducer（seq / health / closeMarks）
        ▼
query_live_connections 默认第一页 → 表格
```

- Rust 仍是权威投影。前端只缓存当前页 DTO 和视图选择。
- Tauri `Channel` 只活在 `lib.rs` 的订阅表里，不进入 `AppFacade`，避免单元测试依赖 WebView。
- 采集节拍函数可在无 Tauri 的 `cargo test` 里驱动。

## 持续采样

新增 `c2/collector.rs`（或同等小模块），导出可测的一次节拍，不在模块里 `spawn`：

1. 读快照（短锁）：`NormalReady`、`collector_running`、非空 loopback 地址、可解析 secret。任一不满足则跳过 HTTP。
2. 释放锁后调用现有 `fetch_connections` + `normalize_snapshot`（与 `connect_tcp` 后半段相同，不复制鉴权规则）。
3. 再取锁：`ingest_snapshot`；失败则 `apply_probe_err`。
4. 把返回的 `MonitorStreamMessage` 交给订阅表。

`lib.rs` `setup` 启动一条后台任务：`sleep(SAMPLE_INTERVAL)` 后调用节拍。间隔使用 C2 常量，默认 1000 ms，不改 C0 峰值定义。

启动条件：

- 进程启动即跑循环。未配置地址或恢复模式时节拍空转。
- `test_controller` 成功后下一拍开始有数据，用户不必再点测试。
- 已持久化地址时，下次启动同样空转直到地址可用；有地址则自动取帧。

停止条件：`collector_running == false`、`disconnect_now`、`RecoveryOnly`、shutdown。循环在 shutdown 后退出，不在暂停时拆掉任务。

锁约定：HTTP 期间不得持有 `Mutex<AppFacade>`。secret 只在短锁内复制到栈上的 `Secret` / 临时 `String`，不写入日志、错误或 Channel。

### 暂停与断开

当前 `apply_lifecycle` 对 `publish(..., Vec::new())` 会把 `hub.rows` 换成空集。暂停不是连接消失。

- `Paused` / `Resumed` / `SleepGap`：更新 coverage 与健康，发布时传入当前 `hub.rows()`，或走已有 `publish_health` 并单独写 snapshot 覆盖字段。不得 `publish` 空行。
- `Disconnected` / `Cancelled`：允许清空当前行。
- `Snapshot`：仍以本帧连接集合为权威。

## Channel 桥

`subscribe_monitor` / `resync_monitor`：

1. 向 facade 取 bootstrap / resync 消息。
2. 把 `Channel` 记入 `subscription_id → Channel`。
3. 发送 bootstrap。

`resync` 删除旧 id。新订阅可覆盖同一 WebView 的旧 Channel。`send` 失败则移除该 id，并 `hub.drop_subscription`。

`ingest_snapshot`、`apply_lifecycle`、`apply_probe_ok`、`apply_probe_err` 改为把 `publish` / `publish_health` 的消息返回给调用方。`lib.rs` 命令与采集节拍负责转发。

无活跃 Channel 时：`hub.set_serialize_ui(false)` 或保持现有 `active.is_empty()` 短路，丢弃 coalescer，不序列化 10k upsert。

本任务按单 WebView、单活跃订阅实现。不修 `publish` 只带第一个 `subscription_id` 的既有限制。

## 前端

新增小模块（建议 `src/ipc/live-session.ts`），`main.ts` 只负责渲染和点击：

- Tauri 环境用 `@tauri-apps/api/core` 的 `Channel` + `invoke('subscribe_monitor', { onEvent })`。预览态（无 Tauri）保持现有 preview bootstrap，不听 `window.message`。
- `onmessage`：`decodeMonitorMessage` → `reduceMonitor`。`needResync` 时调用 `resync_monitor`。
- `bootstrap` 与 `connectionDelta` 之后调用 `query_live_connections`，用返回的 `rows` 替换表格数据（稳定 identity 序由 Rust 保证）。
- reducer 仍维护 `subscriptionId` / `lastSeq` / `closeMarks` / `snapshot`。表格行以查询页为准，避免 Channel upsert 无序。
- 关闭按钮继续走 `close_connection`。

默认查询：

```text
filter: 全空
sortField: identity
descending: false
cursor: null
limit: LIST_PAGE_DEFAULT（200）
```

超过 200 条不翻页。本任务不改 DTO。

## 空态

纯函数（建议 `liveEmptyKind`）输入：

- `settings.address` 是否为空
- `snapshot.health.session`
- `desktop.collector_running`（经已有 `tray_summary`，或 bootstrap 后缓存；暂停后以托盘/lifecycle 为准）
- 当前页 `rows.length`
- `needResync` / `errorZh`

输出四类之一加「有行」。有行时仍显示健康条，不挡住表格。

文案用现有 `HEALTH_ZH`，另补：

- 未配置：去设置页测试连接
- 已连接无行：当前没有活跃连接
- 采集暂停：托盘选择继续采集
- 订阅缺口：重新订阅 / 重载窗口

删除「关闭全部连接入口不存在」。

暂停检测：不要只靠 `health.session === "paused"`（当前 pause 不改 `session_status`）。优先 `tray_summary.collector_running === false`，或 coverage `closed` + `pause_or_shutdown`。

## 测试

Rust：

- 节拍：fixture HTTP 两次不同 snapshot，第二次后 `hub.rows()` 变化；暂停后不再请求（计数器不增）。
- 暂停：先 ingest 有行，再 `Paused`，行仍在。
- Channel：订阅表双发 mock sink，`publish` 后 sink 收到 `connectionDelta`；未订阅时不要求序列化。
- 现有 `query_connections` 乱序 identity 测试保留。

TypeScript：

- `liveEmptyKind` 覆盖未配置 / 未连接 / 已连接无行 / 缺口。
- reducer 现有迟到订阅、缺口、`204` 后 remove 保留。
- 断言产品源码不再监听 `window.message` 作为 Channel。

## 兼容与回滚

- 不改 `MonitorStreamMessage` 字段，不改 SQLite。
- 回滚：停采集循环、`subscribe_monitor` 恢复只发首帧、前端回到静态空表。C1 库与设置保留。

## 取舍

| 选项 | 结论 | 原因 |
|---|---|---|
| WebSocket `/connections` | 不做 | PRD 冻结 HTTP GET；`connect_tcp` 已走 GET |
| Channel 内带全量表 | 不做 | C2 合同：列表走 Command |
| 前端只靠 delta 填表 | 否 | delta 无稳定排序；查询页才是表的权威 |
| 采集循环放进 AppFacade | 否 | 避免把 async/HTTP/Channel 塞进可单测 facade |
