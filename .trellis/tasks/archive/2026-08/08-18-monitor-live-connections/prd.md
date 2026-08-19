# 实时连接无数据分析与优化

## 目标

用户打开「实时连接」后能看到本机控制器当前连接，并随采集更新。表格为空时说明原因和下一步，不得只写「无数据」。

## 背景

用户截图与 `residential-monitor/src/main.ts` `renderLive()` 一致：验收备注 + 七列表 +「无数据」。页面已挂上，不是未交付占位。分层证据见 `research/empty-live-table.md`。

三层断点（仓库可独立证实）：

1. `test_controller` 只经 `ControllerSession::connect_tcp` GET 一次 `/connections`。没有 1 Hz 循环，`collector_running` 未接到取帧。
2. `subscribe_monitor` 发出 `bootstrap` 后丢弃 Tauri `Channel`。`ingest_snapshot` 丢弃 `hub.publish()` 的 `connectionDelta`。
3. 前端未调用 `subscribe_monitor` / `query_live_connections`，却监听 `window.message`。`bootstrap.snapshot` 不含行，且 reducer 会清空 `connections`。

用户已选定范围 **A**：接通数据通路 + 可诊断空态。不补筛选栏、详情抽屉、虚拟化滚动。

## 需求

### R1 根因可复查

- 任务目录保留分层证据：采样、Channel、前端订阅/查询、空态。
- 实施不得绕过 C1 核算，不得把 mihomo 原始 JSON 交给视图。

### R2 持续采样

- 测连成功后，或设置里已有回环地址且采集未暂停时，产品进程按约 1 Hz HTTP GET `/connections`，经现有 `ControllerSession` 归一化后走 `ingest_snapshot`。
- HTTP 失败不得持锁重试到死；按现有 `SessionStatus` 更新健康，并形成 C1 coverage。不得把未采集时段写成零。
- 暂停采集、断开、恢复模式、明确退出时停止取帧。
- 暂停不得把仍存在的连接从投影里抹成空集；断开才允许清空当前行。
- 使用已有 HTTP GET。不新开 WebSocket。TCP 只接受 loopback。secret 只走现有 header / 凭据路径。

### R3 Channel 与查询接通

- `subscribe_monitor` / `resync_monitor` 保存 Channel，后续 `publish` 发到活跃订阅。发送失败则丢掉该订阅。
- 无订阅者时不序列化高频 UI payload；采集与写入继续。
- 前端在 Tauri 环境启动和 WebView 重建时调用 `subscribe_monitor`，用现有 decoder + reducer 消费消息。删除对 `window.message` 的 Channel 误用。
- 进入「实时连接」、bootstrap 完成、收到 `connectionDelta` 后，用 `query_live_connections` 拉取默认第一页（`sortField=identity`，`limit` 用 C2 `LIST_PAGE_DEFAULT`）。不把全量数组放进 Channel。
- 序号缺口或 schema 不兼容时冻结并 `resync`，不猜测状态。

### R4 可诊断空态

实时页用健康、是否已配置地址、采集是否在跑、查询行数和订阅状态区分：

| 状态 | 用户应看到 |
|---|---|
| 未配置控制器 | 说明去设置页填写地址并测试连接 |
| 未连接 / 鉴权失败 / 端点不存在 | 现有中文健康文案 + 下一步 |
| 采集暂停 | 说明可从托盘继续 |
| 已连接且查询为空 | 「当前没有活跃连接」 |
| 订阅缺口 / 协议不兼容 | 停止应用增量，提供重新订阅 |

禁止再用验收句「关闭全部连接入口不存在」当页面说明。页面展示会话健康和最后采样时间。

### R5 列表行为

- 列：域名、进程、主分类、上行、下行、网络、单条关闭。
- 默认按稳定 identity 排序。随机重排输入不得改变同一页身份。
- 只能关闭单条当前连接。`204` 只标「已发送关闭请求」，后续 remove 才标「已关闭」。
- 缺失字段显示「未知」，不填零或伪默认。
- 前端不重做分类、守恒或 Top N。
- 超过默认页大小的连接本任务不提供下一页；不得因此改 Channel 合同。

## 非目标

- 关闭全部连接。
- 改 Clash / mihomo 配置，代理或读取流量内容。
- WebSocket 采集、非 loopback TCP、多控制器。
- 重跑 C2 10k × 30 分钟峰值，或把短时峰值写成 30 天容量。
- 改报告、告警、备份、Retention 或 Recovery Shell 行为。
- 更换产品视觉世界或侧栏壳。
- 完整筛选栏、连接详情抽屉、虚拟化滚动、keyset 翻页 UI。

## 验收标准

- [x] **AC1 根因**：`research/empty-live-table.md` 列出三层断点并带文件锚点。
- [x] **AC2 采样**：测连成功后无需再点测试，约 1 秒节拍更新 hub 行；暂停或断开后停止取帧；暂停保留暂停前投影行。
- [x] **AC3 订阅**：前端调用 `subscribe_monitor`；自动化证明 bootstrap 之后的 `connectionDelta` 经保存的 Channel 到达，而不是 `window.message`。
- [x] **AC4 列表**：`query_live_connections` 返回的行出现在表格；identity 乱序输入时首屏身份稳定。
- [x] **AC5 空态**：未配置、未连接、已连接但无行、订阅缺口四类文案可区分；页面无「关闭全部」入口和验收备注句。
- [x] **AC6 关闭**：单条关闭仍走现有命令；`204` 不直接显示已关闭。
- [x] **AC7 回归**：`npm --prefix residential-monitor run typecheck`、`lint`、`test`、`build` 通过；相关 `cargo test` 覆盖采样节拍、暂停不清空行、Channel 转发与查询页。secret 扫描仍为零。
