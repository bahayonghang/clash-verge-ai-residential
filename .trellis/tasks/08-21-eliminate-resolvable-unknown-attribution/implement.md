# 实施：消除可解析连接的 Unknown 流量归因

## 0. 开工门与基线

- [x] 用户已审阅本任务最终规划摘要，并在后续消息明确批准实施。
- [x] 仅在批准后运行 `task.py start`；实现前加载 `trellis-before-dev` 与 backend/frontend/storage 相关 spec。
- [ ] 记录当前工作树、运行安装包版本和开发态数据库备份/只读统计；不读取或输出 controller secret。
- [x] 保存一份脱敏 `/connections` fixture corpus：完整字段、processPath-only、process 全空、单跳 DIRECT、多跳、空链、metadata 后到/暂失、frame 缺字段/类型错误。真实载荷拿不到时保留 `UNVERIFIED`，先用合成 fixture 锁定契约。
- [x] 把当前 24h 匿名聚合（Host 4.4 GiB、DIRECT 4.7 GiB、Process 13.4 GiB、bundle epoch 1/seq 1..68211）作为迁移前基线，不修改原库。

## 1. P0：修复重启后的静默 Duplicate 与 session epoch

### 1.1 Durable writer epoch

- [x] 在 `residential-monitor/src-tauri/src/storage.rs` 增加原子 `reserve_writer_epoch`，复用 `bundle_epoch` 与 `data_version`；覆盖空库、旧库、多次 boot、溢出/fail-closed。
- [x] 在 `residential-monitor/src-tauri/src/c2/facade.rs` 的 NormalReady boot 中使用新 epoch，`bundle_seq=1`；RecoveryOnly 不声明可写 writer。
- [x] 为生产 `CommitBundle` 生成覆盖实际 frame/slice 的稳定 SHA-256 fingerprint；相同 receipt + 不同 slice 必须 `PayloadMismatch`，不能因空 payload 误报 Duplicate。
- [x] 去掉 `commit_alert_bundle(...).ok()` 的静默失败：Applied/Duplicate、PayloadMismatch、RetryWindowExpired、SQLite error 分别更新 data version、health/coverage 与脱敏日志。

### 1.2 Durable controller epoch

- [x] 复用 `controller_epoch` 为每次成功的 collector generation 分配新 `epoch_id`；`AccountingEngine` 从外部 epoch 初始化，不再每进程从 0 开始。
- [x] 在连接成功/重连、显式 Restarted、全局 meter 回退、可信 `start`/counter reset 信号上结束旧 generation、清 canonical cache、分配新 epoch并以首帧建立 baseline。
- [x] HTTP/JSON 失败或 incomplete frame 只形成 gap/error，不发布“0 connections 完整帧”，不结束全部 session。
- [x] 测试同一数据库两次 `AppFacade::boot`：第二次第一份有效帧立即 Applied、data_version 增长、metadata/facts 可查；相同 raw id 不复用旧 `session_pk`。

检查点 A：只提交 P0 可靠性修复及测试。若本检查点失败，停止后续 Unknown 优化；否则任何新元数据仍可能不落库。

## 2. Controller 边界与 canonical metadata

### 2.1 严格 normalize

- [x] 修改 `residential-monitor/src-tauri/src/controller.rs`：根 meter/connections 字段存在性和类型错误可观测；trim 字符串、过滤空 chain 元素；未知附加字段保持兼容。
- [x] `providerChains` 仅进入诊断存在性计数，不作为 chains fallback，除非真实固定 fixture 后另行修改设计并重新审批。
- [x] 增加无敏感值的字段覆盖计数：host/sniff/IP/absent、process/path-only/absent、chains/provider-only/absent；不记录字段值。

### 2.2 单一 canonical owner

- [x] 在 `residential-monitor/src-tauri/src/accounting.rs`（必要时新建 `session_attribution.rs`）让每个 `epoch:id` 的 active state 同时持有 counters 与 canonical metadata。
- [x] 调整 `c2/facade.rs`：先 merge 完整 snapshot，再从同一 canonical result 生成 live rows 与 accounting facts，删除两条独立 fallback 路径。
- [x] 扩展 `session_host.rs` 或新模块保存 Host 来源等级：explicit host > sniffHost > destinationIP；覆盖 explicit→sniff、IP→sniff→explicit、空帧不降级。
- [x] 新增 `resolve_process_identity`：`process` 优先，缺失时只取 `processPath` basename；覆盖 Windows/Unix 分隔符、目录结尾、空白、超长；完整 path 不进历史维度/日志。
- [x] Chains 只接受非空整组更新；空帧沿用同 generation 已知链；connection generation 变化后不继承。
- [x] 保持首帧 baseline、counter 回退不发明流量、连接消失不猜尾流量。

## 3. SQLite 属性保真与 session 旁证

- [x] 修改 `storage.rs::intern_and_attr`：host/process/rule/network/chain 字段级 non-null merge；Host `host_id` 由 canonical `connection_session.host` 生成，raw 与物化层一致。
- [x] primary category 继续遵循当前策略，不盲目 `coalesce`。
- [x] canonical chain 非空且变化时，以事务内 delete+insert 替换 `connection_chain` 全组；空帧不删；保持 `chain_key` 与 ordered rows 一致。
- [x] 测试 rich→empty、empty→rich、path-derived→direct process、host provenance、new epoch isolation、delta=0 时 late metadata 仍可补全。
- [x] 测试后到 metadata 对同一 session 既有 raw minute 的 session-wide enrichment；明确这不是 per-minute attr history。

检查点 B：运行 controller/accounting/storage 定向测试和只读数据库 round-trip。确认 known metadata 不再被空帧清除，再继续修查询与物化。

## 4. Chain identity、查询与受控重物化

### 4.1 Pure/SQLite helper

- [x] 在 `residential-monitor/src-tauri/src/c3/rule_name.rs` 新增 `chain_identity` 与 SQLite 注册；保留 `last_chain_hop` 的 Rule-only 单跳 None 语义。
- [x] 测试：NULL/空白→None，`DIRECT`→DIRECT，`ProxyA`→ProxyA，`node>group`→group；Rule `DIRECT + IPCIDR` 仍归 IPCIDR。
- [x] 更新 `storage.rs` 所有 reader/writer connection 的函数注册。

### 4.2 Raw/filter/materialization

- [x] 更新 `c3/sql.rs` 的 `RANK_RAW_CHAIN` 与 chain filter。
- [x] 更新 `c3/retention.rs` 的 chain dictionary intern、HOURLY_CHAIN 与 raw/hourly key parity oracle；测试必须对外部期望断言，不能只让两层“一致地错误”。
- [x] 确认 Rule raw/hourly grouping、residential classification 和 chain 下钻无回归。

### 4.3 派生层修复

- [x] 新增版本化 chain materialization marker/watermark，不改旧 migration checksum。
- [x] 对仍有 raw 的窗口：事务性删除旧 hourly chain 行→intern→重建 hourly→删除并重建受影响 daily chain→守恒验证→写 marker。
- [x] 失败 rollback；不触碰 Host/Process old missing，不改成功 `report_archive`；raw 已删除区间保持旧 Unknown。
- [ ] 用当前开发态库副本验证：约 4.7 GiB 从 `__unknown__` 精确进入 `DIRECT`，总 upload/download 不变且没有旧 0 行双计。禁止直接修改用户原库做试验。

检查点 C：Chain raw、hourly、daily、filter 四条路径统一，Rule non-regression 和迁移守恒通过。

## 5. Historical attribution quality DTO

- [x] 在 `residential-monitor/src-tauri/src/c3/query.rs` / `service.rs` / `sql.rs` 增加 exact `AttributionQuality`：known/missing upload/download/connections + complete/partial/unavailable；按 grouping 使用同一 identity 规则。
- [x] 硬断言 `known + missing == totals`；coverage gap 与 attribution missing 分开。
- [x] `ReportResult` 与 Rust serde DTO 同步输出；ranking missing row继续保留 `__unknown__` 和字节。
- [x] 在 `residential-monitor/src/dto.ts` / `ipc/decoder.ts` 完整解码 rankings 和 attributionQuality，拒绝缺字段、非法 enum/负数/非有限数；删除 unsafe cast 路径。
- [x] 测试 process 全缺、少量已知、完整已知；TopN 不得改变 quality 总量。

## 6. Live observation phase 与 UI 语义

### 6.1 Backend/IPC

- [x] 在 `c2/hub.rs` / DTO 中增加 `observationPhase`：unconfigured、connecting、baselinePending、current、paused、disconnected、resyncRequired、decodeFailed。
- [x] current 之前的 active count 不作为真实 0；可保持字段 number，但前端必须按 phase gate，或同步改为 nullable 并给 reason。
- [x] Overview decoder 对每个约定字段使用 `hasOwn`：present/null 合法，missing 解码失败。

### 6.2 React

- [x] `components/features/overview/caliber-grid.tsx` / `caliber-card.tsx` 按 phase 显示“等待控制器 / 建立基线 / 当前不可用 / stale / current”，只在 current 把 0 当真实 0。
- [x] Overview 明示 `实时 · 当前控制器` 与 `历史 · 已存储数据 · 时间窗`；connecting 时历史趋势/Top 继续显示并保留 report coverage。
- [x] `top-columns.tsx` / dimension page 显示 attribution quality；unknown sentinel 使用维度化 label，Process 显示已知/未报告覆盖，而不是普通 Unknown 第一名。missing bytes 仍可见且守恒。
- [x] `live-empty.ts` 把 connecting 与 disconnected 分开；断连保留旧 rows 时标 stale，rowCount 不得优先冒充 current。
- [ ] 更新中英文 i18n，键集合测试保持一致；四主题和窄/宽窗口不溢出。

## 7. 文档与 Trellis spec

- [x] 更新 `residential-monitor/docs/reporting.md`：四类 Unknown、session-wide enrichment、Chain identity、process path basename、历史不可恢复与 frozen archive 边界。
- [x] 更新 `.trellis/spec/residential-monitor/backend/modules-and-errors.md`：durable writer/controller epoch、canonical metadata owner、strict complete-frame 语义。
- [x] 更新 `.trellis/spec/residential-monitor/storage/sqlite-contract.md`：字段级 merge、chain identity scalar、重物化事务与守恒。
- [x] 更新 `.trellis/spec/residential-monitor/frontend/dto-and-decoding.md` / `view-state.md`：observationPhase、strict present-null decoder、双数据面、attributionQuality 和维度化 missing label。
- [x] 检查旧 spec 中“所有 Unknown 不可下钻”等已漂移表述并统一，不做无关文档清理。

## 8. 验证命令

### 定向后端

```powershell
rtk cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml writer_epoch
rtk cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml controller_epoch
rtk cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml metadata
rtk cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml chain_identity
rtk cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml c3::
```

测试过滤名按实际新增测试命名调整；每条必须有至少一个匹配测试，不能把 `0 tests` 当通过。

### 前端与全门

```powershell
rtk npm --prefix residential-monitor run typecheck
rtk npm --prefix residential-monitor run lint
rtk npm --prefix residential-monitor test
rtk npm --prefix residential-monitor run build
rtk just monitor-check
rtk just ci
rtk git diff --check
```

### 数据迁移/守恒 smoke

- 在临时副本运行 chain repair，前后记录 raw/hourly/daily upload+download；必须完全相等。
- 断言当前 24h `DIRECT` 约 4.7 GiB，Chain `__unknown__` 只剩真正空链（本基线应为 0）。
- 断言旧 Host 4.4 GiB 与旧 Process missing 保持守恒，不被改名为已知。
- 模拟旧 receipt DB 重启，第一帧 data_version 立即增长。

## 9. 真实控制器与安装态验收

以下在执行前均为 `UNVERIFIED`，自动化测试不能替代：

- [ ] 确认运行 binary/build SHA 包含本任务实现；禁止用旧安装包截图验收新代码。
- [ ] 在同一新时间窗保存脱敏字段存在性统计，对比 Clash Verge 与 monitor 的 raw id、Host、Chain、Process；不记录 secret、完整 host/IP/path。
- [ ] 验证 `["DIRECT"]` 在 live/Top Chain/Chain page 都显示 DIRECT。
- [ ] 验证 process/processPath 都缺时显示“控制器未报告”与覆盖量，不伪造应用名；若有 path-only，验证只显示 basename。
- [ ] 验证重启应用后第一份新 delta 立即进入数据库，Host fallback 可在新窗口生效。
- [ ] 验证 connecting/baseline/current/disconnected/stale 视觉状态；历史 24h 数据在 live 断连时仍明确作为历史可读。
- [ ] 验证四主题 × 中英文 × 最大化/窄窗口；键盘、屏幕阅读器、reduced-motion 无回归。

## 10. 风险与回滚点

| 风险文件/机制 | 失败模式 | 控制 |
| --- | --- | --- |
| `storage.rs` epoch/receipt | 重复写、跳写、旧库无法打开 | 检查点 A、旧库 fixture、kill-point/rollback tests |
| `accounting.rs` generation | 跨连接串 metadata 或过度切 epoch 丢差分 | counter/start/reconnect矩阵；保持 observed lower bound |
| `intern_and_attr` | 将可撤销 category 错误永久保留；历史全被 last value 重标 | 维度特定 merge，不对 category 通用 coalesce |
| Chain helper | 改坏 Rule grouping | 独立函数 + DIRECT/IPCIDR oracle |
| 重物化 | 旧 0 行残留造成双计 | delete+rebuild transaction + before/after totals |
| processPath | 泄露用户路径 | 只存 basename；日志/fixture 扫描 |
| DTO 收紧 | backend/frontend 漂移导致白屏 | 同提交升级 + decoder contract tests + Recovery shell |
| UI 状态 | 把历史 ready 错绑 live connected | 双数据面集成测试 |

实施提交建议按检查点 A/B/C/UI 分开，便于回滚和审查。不要在本任务中自动修改 Clash/Mihomo 配置、清空用户数据库、重建冻结档案或推送远端。

## 11. 实施证据（2026-08-21）

### 自动化 PASS

- 检查点 A/B/C 合并后端与最终审查修复：`cargo fmt --check`、`cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings` 通过；最终 `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace` 合计 300 passed / 1 ignored（lib 297 passed / 1 ignored，kill gate 3 passed），其他 targets 与 doc tests 无失败。
- Chain repair 合成临时库：marker 首次写入且二次幂等；注入 trigger 的失败事务完整 rollback；成功后 raw/hourly/daily upload=30、download=90 守恒，旧 DIRECT `dimension_id=0` 行消失且新 DIRECT id 非零。
- 前端：最终 `npm --prefix residential-monitor run check` 通过 typecheck、lint、Vitest 54 files / 194 tests 和 Vite build（2461 modules）。
- 仓库安全门：`just monitor-check`、`just ci`、`git diff --check` 与 Trellis context validate 全部通过；根测试 74 passed，secret scan 通过。两个 Just recipe 中的本地 lockfile 安装均报告 0 vulnerabilities；npm 仅提示 `esbuild` install script 尚未列入 `allowScripts`，未造成门禁失败。
- 定向后端过滤均实际命中并通过：writer epoch 1、controller epoch 2、metadata 6、chain identity 4、`c3::` 86、SQLite probe 3；旧 frozen archive 缺 `attributionQuality` 的兼容 fixture 1 passed。
- 最终审查另外收紧三处失败边界：必填 connection row 的 over-limit/重复 id 整帧拒绝；旧 Chain 派生量与现存 raw 不等时回滚且不写 marker；前端拒绝与 known/missing 计数不一致的 attribution status。

### 保持 UNVERIFIED / 未改动

- 未抓取真实 Mihomo `/connections` body，未读取 controller secret；真实字段覆盖只提供新诊断计数，不能用合成 fixture 声称现场比例。
- 未修改或复制用户原始数据库。开发态数据库副本上的约 4.7 GiB DIRECT 修复 smoke 尚未执行；Host 4.4 GiB 与 Process 13.4 GiB 旧缺失未被推断或回填。
- 未构建/安装 native 包，未核对运行 binary SHA，未执行真实 controller、四主题、中英文、最大化/窄窗、键盘/屏幕阅读器/reduced-motion 人工验收。
- Windows Credential Manager 测试保持 ignored，因为它会写入本机凭据存储，需独立人工授权。
