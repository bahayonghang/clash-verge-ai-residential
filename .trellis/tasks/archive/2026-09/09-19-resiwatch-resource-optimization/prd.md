# 优化 ResiWatch 后台性能与资源占用

## Goal

降低常驻监控的 CPU、内存与磁盘写入，使过期明细按既有保留周期退出；保持采集核算、告警、历史汇总、报告和故障恢复正确。采用一个协调任务，按基线、热路径、保留、集成验收顺序推进。

## Authority and status

2026-09-19 用户请求分析截图并创建优化任务，随后明确选择「允许汇总并校验守恒后清理过期明细」。沿用 raw 默认30天/上限90天、精确维度396天，不缩短既有期限。

2026-09-19 用户明确回复「审阅完毕，开始实施」，已批准本规划并进入 `in_progress`。授权产品代码、隔离fixture与本地验证；不包含替换安装包、停止现有采集、清理用户数据库、自动VACUUM、提交或发布。

## Background

详见 [research/diagnosis.md](research/diagnosis.md)，以下均不是优化后的结果。

| 证据 | 已确认事实 | 限定 |
| --- | --- | --- |
| 截图 | PID25188，累计CPU5872秒，DB约1.14GB，spool持续写入，未签名 | 累计CPU不等于持续满核；未签名不能解释资源消耗 |
| 安装态 | 10秒样本单核CPU3.43%；另30.60秒样本16.75%，写I/O73,550,314B/20,814次；native工作集约59–61MiB | 窗口和负载未控制；I/O不等于物理磁盘写入；未包含WebView |
| SQLite | minute368万、session264万、chain506万、receipt164万；主库约1.17GB，freelist仅1页 | 分别取样；无dbstat，表/索引字节占比未知 |
| 数据范围 | 约21.10天；hourly/daily维度均0，档案720小时+396日均ok | 尚未到默认30天，空汇总层不能单独判故障；不会立即清掉这21天数据 |
| 版本 | 安装exe修改时间09-02；仓库09-04已修复逐行prepare等问题 | 安装精确commit未知，不能把源码判断直接等同安装态根因 |

## Requirements

以下代码路径均以 `residential-monitor/` 为前缀；完整来源见研究文件。

| ID | Priority | Requirement and evidence |
| --- | --- | --- |
| R1 | P1 | 建立包含真实facade提交、档案调度、报告的同负载AB基线，分开native/WebView、CPU、文件I/O、DB/WAL/freelist、内存和延迟。旧replay使用简化提交（`src-tauri/src/bench.rs:192`），不足以代表产品热路径。 |
| R2 | P1 | 消除每tick完整档案扫描/时间范围重算和未变metadata重写；失败档案有界重试且不饿死积压。保持facts/coverage/alerts/outbox/receipt同事务（`src-tauri/src/lib.rs:292,317`、`src-tauri/src/c3/archive.rs:120`、`src-tauri/src/storage.rs:697`）。 |
| R3 | P2 | 内部告警/档案不生成即弃spool；可见视图限制重叠查询，隐藏窗口停止展示性刷新，恢复合并刷新一次，采集告警继续。依据`src-tauri/src/c4/period.rs:118`、`src-tauri/src/lib.rs:338`、`src/hooks/use-live-page.ts:223,279`、`src/app.tsx:85`。保留公共token复用/TTL/配额语义。 |
| R4 | P1 | 实际守恒后分块物化、清理过期minute/session/chain/attr，以及必要的receipt/coverage/无引用字典生命周期。当前DELETE关闭（`src-tauri/src/c3/query.rs:21`），verify_layer仅哈希区间文字（`src-tauri/src/c3/retention.rs:898`），raw删除只删minute（同文件`:924`），receipt无时间字段且过期判断依赖剩余行（`src-tauri/src/storage.rs:130,352`）；先补安全机制再启用。 |
| R5 | P1 | 如实表达raw/汇总查询能力，超期精细查询返回不支持/未知；区分可复用页和已释放文件字节。保留FULL durability、crash/retry幂等、取消、低空间及RecoveryOnly保护，实测通过后更新规格文档。 |

## Acceptance Criteria

| ID | Mapping | Observable acceptance |
| --- | --- | --- |
| AC1 | R1 | 同机器/配置/输入/数据库记录baseline和candidate SHA/哈希、活动数、变化率、窗口状态；覆盖A=50/250/1000、无变化/metadata变化/计数全变化、档案齐全/积压/失败。monitor-bench真实链路记录CPU、native+WebView内存、文件写I/O、spool、DB/WAL/freelist和commit/query p50/p95/p99，区分冷/热查询。 |
| AC2 | R2 | 3600个1Hz虚拟tick测试在未到下一到期点且档案齐全时，不重复历史级范围生成和全扫描；仅启动/到期/有界重试触发。失败不无限重试或阻塞其它作业。跨日/DST、时钟跳变、重启、暂停恢复正确。 |
| AC3 | R2 | 后续metadata完全相同帧对attr/chain实际修改数为0；新会话、host升级、non-null merge、整组chain替换、policy变化和generation重置正确。失败/未知commit后同bundle重试不遗漏metadata或重复核算。 |
| AC4 | R3 | 内部period/archive临时spool写入为0，结果与错误/取消语义保持。隐藏后稳态展示查询/token写入为0；恢复只合并刷新一次，采集不间断。每视图最多1个查询在途并合并最新意图；600s TTL、8 token、单32MiB/总128MiB及导出回归通过。 |
| AC5 | R4,R5 | 每个关闭UTC chunk的raw→hourly→daily/core upload/download按分类与各维分别守恒，coverage并集/gap正确；count/duration按正式定义对照完整raw。一个session跨多个小时的daily计数不得重复；整日精确聚合最终确认前，该日任何raw小时不得删除，最后小时失败/中断仍保留该日raw。破坏任一维或coverage必须校验失败、删除0行、水位不推进。取消/busy/低空间/崩溃不留半chunk，重复清理不能用残缺raw覆盖完整汇总。 |
| AC6 | R4 | 跨30天/396天fixture中，过期raw及无引用已关闭session/chain/attr退出，活跃/重试引用保留。升级fixture含当前schema的NULL-ended历史会话：有持久退役epoch证据且无引用者可回收，活跃零流量及证据不全者保护并单列，不能只测新会话或伪造结束时间。receipt保留最近24小时与100000条并集，持久过期边界先于裁剪，窗外重复永远拒绝。覆盖空epoch、restart、backup/restore、unknown commit；45天及更长模拟证明分层占用符合期限模型，长期daily core及仍保护的旧存量单列。 |
| AC7 | R1–R5 | 待验证性能目标：A=250、1Hz、metadata不变且档案齐全，预热后3轮各5分钟，相对同源baseline CPU秒下降≥30%、归属应用文件的写入字节下降≥50%；变化场景commit/query p95不恶化超过10%，页面2秒/报告10秒deadline保持。未达目标不能只凭测试绿宣布完成。 |
| AC8 | R1,R5 | 完整30天A=50/250/1000容量门、10k/1Hz/30分钟峰值、24h安装态soak和`just ci`通过；短smoke不能代替。native+WebView总Private Bytes p95≤同场景基线110%，预热后内存/句柄/队列/token/WAL不持续无界增长。保留durable commit p95<1.5s、正常最大<3s、队列不持续>2帧既有门；手动VACUUM在fixture证明回收和完整性，不自动操作用户库。 |

## Out of scope

- 路由脚本、Clash/Mihomo配置、凭据/控制器调整、签名采购或发布。
- 清空历史、缩短期限、近似Top K、跳过核算事件、削弱FULL同步。
- 新依赖、通用调度/缓存平台、自动VACUUM、自动替换安装程序。

## Decisions and remaining evidence

无阻塞产品问题：用户允许守恒后的过期明细清理，沿用原保留策略。安装构建归属、表/索引字节、真实CPU profile、数据安全回归及长期/安装态AB证据留待实施阶段完成。现有21天仍在30天raw期内，不承诺1.17GB立即大幅缩小。
