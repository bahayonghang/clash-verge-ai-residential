# 实施计划：接通实时连接数据通路

## 启动前门禁

- [x] 用户已批准本任务最新规划摘要。未经该批准不运行 `task.py start`，不改产品代码。
- [x] 实施前读 `.trellis/spec/residential-monitor/{backend,frontend}/index.md` 及其 checklist。
- [x] 不改 Channel / Command JSON 形状，不改 C1 migration。

## 执行顺序

### 1. 采样节拍

- [x] 在 `c2/contract.rs` 增加采样间隔常量（1000 ms），不改 `LIST_PAGE_*`。
- [x] 新增可单测节拍：短锁读配置 → HTTP GET `/connections` → `ingest_snapshot` / `apply_probe_err`。
- [x] `ingest_snapshot` / `apply_lifecycle` / probe 路径返回要转发的 `MonitorStreamMessage`。
- [x] `Paused` / `Resumed` / `SleepGap` 发布时保留当前行；`Disconnected` 才清空。
- [x] `lib.rs` `setup` 启动后台循环；HTTP 期间不持 facade 锁。
- [x] `test_controller` 成功后不另开第二条循环。

**Gate**：fixture 下连续两拍行变化；暂停后请求计数停止；暂停后行仍在。

**回滚**：删除循环与节拍模块；facade 恢复忽略 `publish` 返回值。

### 2. Channel 转发

- [x] `lib.rs` 增加订阅表：`subscription_id → Channel<MonitorStreamMessage>`。
- [x] `subscribe_monitor` / `resync_monitor` 存 Channel 并发送 bootstrap。
- [x] 采集与 probe 路径把消息 `send` 到仍存活的 Channel；失败则 drop。
- [x] 无订阅者时不序列化高频 upsert。

**Gate**：mock / 单元测试证明 bootstrap 之后的 delta 到达已存 Channel。

**回滚**：命令恢复只发首帧。

### 3. 前端订阅与查询

- [x] 新增 `src/ipc/live-session.ts`（或同等）：`Channel` + `query_live_connections`。
- [x] `main.ts` 删除 `window.message` Channel 误用。
- [x] bootstrap / `connectionDelta` / 进入 `live` 路由后拉默认第一页并渲染。
- [x] `needResync` 调用 `resync_monitor`。
- [x] 预览态（无 Tauri）保持空表 + 可诊断空态，不伪造行。

**Gate**：TS 测试覆盖空态函数；源码不再把 `window.message` 当 Channel。

### 4. 空态与文案

- [x] `liveEmptyKind` + 健康条：未配置 / 未连接 / 暂停 / 已连接无行 / 订阅缺口。
- [x] 删除「关闭全部连接入口不存在」和单独的「无数据」兜底。
- [x] 关闭列与 `204` 语义保持不变。

**Gate**：AC5 四类文案有单测或渲染夹具。

### 5. 质量门

- [x] `npm --prefix residential-monitor run typecheck`
- [x] `npm --prefix residential-monitor run lint`
- [x] `npm --prefix residential-monitor test`
- [x] `npm --prefix residential-monitor run build`
- [x] 相关 `cargo test`（collector / lifecycle rows / channel 转发 / query identity）
- [x] 需要时 `cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --lib -- -D warnings`
- [x] 不跑 `tinstall`、本机 Credential Manager 真机写、登录自启动。

## 风险文件

| 文件 | 风险 |
|---|---|
| `residential-monitor/src-tauri/src/lib.rs` | 锁跨 await、Channel 生命周期、双循环 |
| `residential-monitor/src-tauri/src/c2/facade.rs` | 暂停误清空行；改变 `publish` 返回值漏转发 |
| `residential-monitor/src/main.ts` | 整页 `innerHTML` 重绘丢失焦点；重复订阅 |
| `residential-monitor/src/ipc/reducer.ts` | bootstrap 清空 map 后未再查询 |

## `task.py start` 前检查

- [x] `prd.md` 无阻塞开放问题。
- [x] `design.md` 与 `implement.md` 已写。
- [x] `implement.jsonl` / `check.jsonl` 已换成真实 spec/research 条目。
- [x] 用户已批准本摘要。
