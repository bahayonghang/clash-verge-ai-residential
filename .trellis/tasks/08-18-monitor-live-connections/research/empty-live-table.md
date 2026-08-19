# 实时连接「无数据」根因

依据：2026-08-18 用户截图 + 仓库当前实现。未连接用户本机 Clash Verge 控制器做真机采样。

## 用户可见现象

「实时连接」页渲染了产品表格，不是未交付占位。

- 说明文案：`列表按稳定 identity 排序。关闭全部连接入口不存在。`
- 表头：域名 / 进程 / 主分类 / 上行 / 下行 / 网络 / 操作
- 表体：`无数据`
- 页面不展示会话健康、最后采样时间、活跃连接数或恢复动作

该文案来自 `residential-monitor/src/main.ts` `renderLive()`，不是运行时诊断。

## 数据通路（设计）

```text
mihomo /connections
  → ControllerSession 取帧
  → AccountingEngine.project_live
  → MonitorHub.rows + publish(connectionDelta)
  → subscribe_monitor Channel + query_live_connections
  → TypeScript reducer / 列表
```

C2 合同（`.trellis/spec/residential-monitor/frontend/dto-and-decoding.md`）：

- `bootstrap.snapshot` 是概览 DTO，不含连接数组
- 列表与详情走 `query_live_connections` / `get_connection`
- Channel 只推 `connectionDelta | healthChanged | summaryChanged | alertChanged`

## 断点 1：产品进程没有持续采样

`ControllerSession::connect_tcp` 在 `test_controller` 里对 `/version` 和 `/connections` 各 GET 一次，然后 `apply_probe_ok` → `ingest_snapshot`。

仓库内没有 1 Hz 循环、没有 WebSocket `/connections` 订阅、没有把 `collector_running` 接到实际取帧。

`pause_collector` / `resume_collector` 只改桌面标志并写生命周期输入，不会再拉连接。

结果：即使用户测连成功，hub 最多留下一帧快照；应用重启或未测连时 `hub.rows()` 为空。

锚点：

- `residential-monitor/src-tauri/src/session.rs` `connect_tcp`
- `residential-monitor/src-tauri/src/lib.rs` `test_controller`
- `residential-monitor/src-tauri/src/c2/facade.rs` `apply_probe_ok` / `ingest_snapshot`

## 断点 2：Channel 只发首帧，增量无处可去

`subscribe_monitor` 调用 `hub.subscribe()`，用 Tauri `Channel` 发送一条 `bootstrap`，然后命令结束，Channel 未被保存。

`MonitorHub::publish` 会构造 `connectionDelta`，但返回值在 `ingest_snapshot` 里被丢弃（`let _ = self.hub.publish(...)`）。`active` 只记 `subscription_id → bool`，不持有 Channel。

`set_serialize_ui` 存在，产品路径未使用。

锚点：

- `residential-monitor/src-tauri/src/lib.rs` `subscribe_monitor` / `resync_monitor`
- `residential-monitor/src-tauri/src/c2/hub.rs` `subscribe` / `publish`
- `residential-monitor/src-tauri/src/c2/facade.rs` `ingest_snapshot`

## 断点 3：前端未订阅、未查列表

`main.ts` 从未调用 `subscribe_monitor`、`resync_monitor`、`query_live_connections`、`get_connection`。

它监听 `window.addEventListener("message")`。这不是 Tauri 2 Channel。Rust 不会把 `MonitorStreamMessage` 发到该事件。

`get_bootstrap` 只写入 `state.snapshot`（概览）。`reduceMonitor` 在 `bootstrap` 时把 `connections` 置为空 Map。`renderLive` 只读 `state.connections`，因此稳定渲染「无数据」。

锚点：

- `residential-monitor/src/main.ts` `renderLive`、`main()` 末尾
- `residential-monitor/src/ipc/reducer.ts` `reduceMonitor` bootstrap 分支
- `residential-monitor/src/ipc/decoder.ts` `decodeMonitorMessage`

## 空态不可诊断

实时页不读 `overview.health.session` / `activeCount` / `lastSampleUtc`。

用户无法区分：未配置、测连失败、采集未启动、已连接但当前无连接、订阅缺口。

C2-R4 要求这些状态有独立中文说明和恢复动作。当前实现不满足。

## 已排除

- 侧栏认页错误：截图选中「实时连接」，DOM 走 `renderLive`。
- `visibleRows` 切片错误：空 Map 的窗口仍为空。
- 关闭全部入口：产品禁止关闭全部；文案是验收备注，不是过滤条件。
- 前端分类算法：列表只格式化 DTO。

## 结论

空表首先是产品数据通路未接通，不是筛选或 identity 排序把行藏掉。

要让「实时连接」显示当前连接，三层都要补：持续采样、Channel 常驻并转发 publish、前端按合同订阅并用 `query_live_connections` 填表。只修其中一层，表仍会空。

范围已定为 A：接通上述通路并区分空态。不做筛选栏、详情抽屉、虚拟化滚动。
