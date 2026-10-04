# T05 正式执行器预审

日期：2026-09-30。审查范围：`run-formal-replay.py` 初版、原始 replay 驱动与统计脚本、`bench/facade.rs`。本文件记录启动前问题；修正后的结论另行追加。审查未执行 runner、产品测试、build 或 benchmark，未打开数据库。

## 参数与统计口径

初版保留原 11 对 matrix 和 3 对 primary，合计 28 次 native 调用。每对顺序为 baseline → candidate；primary 保留 r1/r2/r3。

| 集合 | 场景 | 固定参数 | 判定 |
| --- | --- | --- | --- |
| matrix 前 9 对 | A50/250/1000 × unchanged/counters/metadata，complete | hz=1，warmup=5，duration=30，query_every_frames=5，metadata_change_percent=100 | 与原驱动一致 |
| matrix 后 2 对 | A250 metadata，backlog/failed | 同上，额外 period-rule | 与原驱动一致 |
| primary 3 对 | A250 counters，complete | hz=1，warmup=30，duration=300，query_every_frames=0 | 与原驱动一致 |
| 全部 | seed=20260919，start_utc=1800001800 | 不使用 virtual-time；每次新空目录，JSON 在目录之外 | 与原合同一致 |

原 `archive/2026-09/09-19-resiwatch-resource-optimization/research/summarize-isolated-bench.py:45–51` 对三轮 CPU 和 SQLite xWrite 先分别求和，再算相对下降。新 runner 的 sum ratio 与该口径一致。每轮原始值与比值仍需保留；全部应用文件门不能由 SQLite 子集替代。

## 启动前问题

### P1：WAL/FULL 未由原 writer 的实际状态证明

初版只检查顶层 `synchronous == FULL`。当前 `facade.rs` 原字段为硬编码，inventory 没有 journal_mode/synchronous。因此不能将字符串检查写成实际 WAL/FULL 验证。

主会话已批准在既有 inventory 的原 `state.storage.connection()` 上增加只读 PRAGMA 字段。初始 inventory 在 write_before/process_before 之前，最终 inventory 在 wall/process_after/VFS 采样之后。两端检查实际 journal_mode=wal、synchronous=2，顶层旧字符串由最终实际值映射。相同仪表必须进入 baseline 和 candidate，新构建身份不得覆盖旧证据。不得使用重开连接的 synchronous 证明原 writer。

### P1：F12 缺非空生产证据却可能整体 PASS

初版可在 11 个首 reader 比值都不超过 1.10 时返回 0。`facade.rs` 只检查 report totals 非负，JSON 没有每次报告行数、窗口输出与独立 oracle；fixture_hash 只由参数生成。六个 reader 样本不证明窗口非空或报告语义正确。

保留原窗口比值与失败，不修改原参数。没有与最终身份绑定的非空生产 oracle 时，独立 F12 非空门须为 UNVERIFIED。原矩阵比值只能代表该驱动窗口。一次已预读缓存的非空阶段样本不能替代完整 F12、21 次容量或物理冷读。

### P1：数值类型和样本计数可产生假 PASS

初版 ratio 接受负 candidate、Python bool，以及正无穷 baseline。负比值或由无限 baseline 得到的 0 可以落入 PASS。primary 汇总前未逐值验证；零样本 p95 与缺失值也应独立拒绝。

两端必须为有限非 bool 数，baseline > 0、candidate >= 0。primary 各轮先通过数值验证才求和。matrix 的 ingest/private 应各有 30 个样本，primary 应各有 300 个；matrix first/repeated 各 6 个，primary 各 0 个。缺失或非法指标不能转换为 0 或 PASS。

### P2：输出与最终构建身份绑定需补全

需验证 kind/schema_version、source_revision、dir、平台、PID、options 和窗口。错误 JSON 类型、缺键、非零 native exit 必须写入失败收据和最终 summary。初版已逐项保存原始 stdout/stderr、exit，并串行完成其余固定场景；不能只看最后一个 native exit。

正式 candidate 目录必须实际包含 runner 指定的 `candidate-source-before.json`，与 after manifest 的路径集合、值及最终 exe 相符。早期 lazy 目录只有 `candidate-source.json`，不能直接当成最终正式输入。baseline 的更新仪表、源码 manifest 和新 exe 同样必须绑定。

## 失败归属

c278 baseline 与当前 candidate 的正式门失败，证明该成对身份未达到门槛。该差异含既有产品变化，不能直接归因于本轮 lazy cache 或 MinuteSet。认定新算法导致回归还需对应的改动前身份、相同输入和测量边界。回退实验优化时保留独立 Network unknown 合同修复和全部原失败收据。

## 当前状态

原参数、AB 顺序与三轮合计口径审查通过。实施 owner 正在修正执行器与最小计量字段，正式启动确认仍待最终 diff、构建身份和验证收据。完整性能门尚未通过。

## 修正复审（2026-09-30）

本次审查 runner SHA256：`6c88acca5994c02266f262fbc185ebf9d09de1ab9ec96baa302993a0cbf381e9`。修正后的执行器机制审查通过。MinuteSet 实验已建议回退，正式输入仍须绑定回退后的新 candidate 身份；不能直接使用实验目录完成正式验收。

- actual writer：facade inventory 在同一 writer 上读取 journal_mode/synchronous。initial 在 CPU/VFS 起点前，final 在 wall/process_after/VFS 终点后；只读 PRAGMA 没有进入当前测量区间。顶层 synchronous 由最终实际值映射。runner 同时验证初末 wal/2 与顶层 FULL。
- 数值：`number` 只接受有限、非负的 int/float，排除 bool。ratio 另要求 baseline > 0。primary 先逐值检查才求和，null 保持 UNVERIFIED。
- 形状与身份：验证 schema/kind、Windows、PID、全部 options、source_revision、dir、exe hash、fixture hash形状、时区类型。非法 JSON 顶层和缺字段写入 validation_errors。
- 样本：ingest/private/working-set及raw sample数量按30/300核验；reader按6/0核验；p50/p95/p99/max均检查数值。
- F12：原11个窗口比值保留，另设非空生产oracle为 UNVERIFIED，使比值全部通过时也不返回总体PASS。固定start为整分钟，查询end只有start+10/15/20/25/30/35秒；当前query原样保留窗口，service将两端div_euclid(60)，当前Raw分钟范围为空。该静态结论不代替实际报告语义验收。
- 身份：baseline/candidate各自验证before/after manifest相等、manifest文件hash与exe identity绑定、每个源码字节数/hash以及exe hash。保留原始exit、gzip日志、结果与总summary；失败退出2，缺证退出3。

实施代理的 `test-formal-replay-runner.py` 仅提取三个纯验证函数，未启动正式负载。五个测试覆盖合法matrix/primary、错误形状/缺键、零计数/身份错误、bool/负数/NaN/Inf及无效ratio，`formal-runner-validation` 收据退出0。审查代理只读测试与收据，未重复执行。

`candidate-final-20260930` 的baseline manifest为103项、candidate为108项，before/after分别相同。相对初轮完整manifest，baseline只改变facade仪表，candidate只改变facade与raw_fold；路径集合没有增删。四个exe hash与manifest绑定全部匹配，详见 `review-minute-experiment-validation.json`。该目录保留MinuteSet实验，后续回退不得覆盖这些身份。

WAL/FULL实际字段修复、执行器验证修复均只解决证据可靠性。正式F1–F12、AC7全部应用文件写入与完整容量门仍待各自收据，不能以执行器测试通过宣布性能目标通过。

## 保留候选准入（2026-09-30）

MinuteSet已回退，当前有效输入为 `candidate-retained-20260930`。baseline/candidate源码103/108项的before/after、当前源文件及四个exe hash绑定全部核对通过；candidate源码manifest为 `860165F3183AC8BFAAC2D74AF6F29E793575A91A82B1BB23D86EE975BE5C0462`。原writer初末实际WAL/FULL的双边A8实时smoke通过。详见 `review-retained-candidate-validation.json`。runner本身未改变，正式准入通过。

实施owner已启动 `formal-matrix-20260930`。准入通过不代表性能门通过。正式矩阵、主场景及后续容量结果分开终审，全部应用写入、A1000前置条件和安装态证据分别保留未完成状态。
