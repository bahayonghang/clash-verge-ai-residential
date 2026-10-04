# T05 后续阶段前置方案（待强模型审查）

日期：2026-09-30。适用工具：Claude Code、Codex、Grok Build、Kimi Code、OMP。

原 T05 已批准 `raw_fold.rs`、`sql.rs`、`service.rs`、必要 bench 计量和本任务 research 范围。只读阶段归因与上述范围内的最小候选仍有用户授权；后续执行需要先补足归因证据并通过强模型阶段方案审查，无需重复请求同一用户许可。当前正式测量按既定顺序继续，本文件不自动启动新算法。

系统级 trace、安装态、schema、调度器、超出清单的新增文件或公开语义改变需要追加范围批准。A1000 生成仍受已约定的阶段前置条件限制。强模型负责归因、设计、SQL/核算审查和验收判断；较低成本模型只执行已定义的测试 fixture、固定收据整理与格式检查。原生 harness 能力不扩展授权。

## 事实与缺口

1. lazy 字典和 chain 派生缓存的两对 A250 已读缓存对比：all projection 中位下降 15.40%，residential 下降 10.50%；对应 scan 变化为 +0.75% 与 -0.94%。局部结果不证明 A1000。
2. MinuteSet 连续值实验只得到约 1.35% 的名义 scan 降幅、单次整报告 0.18% 变化；每集合增加 16 B。实验已回退，语义测试与失败/通过收据保留。
3. 保留身份的 A250 默认 30 天 all 生产报告为 6735.2968 ms，精确 totals 与 720 行 series 通过。这是已读缓存单次证据；21 次正式容量、首次物理冷页及 A1000 各有独立边界。
4. 长窗投影仍读取全部会话；分钟扫描仍对每行执行 session HashMap 查找及 totals/missing/series/group/exit 累加。当前计量只区分 projection 与 scan，尚未量化 SQLite 行读取、session 查找、group/exit HashMap 和集合操作各自占比。
5. A1000 完整库本轮不生成。原 09-24 PRD R3 与本任务测量前置要求尚未满足。A1000 为 NOT_RUN；保留完整 30 天、host/top20 21 次、每次 exit 0 且整体 wall≤10000 ms 的要求。A250 四倍外推不构成实测，也不证明 A1000 不可能通过。

## 优先步骤与文件

| 优先级 | 拟改或新增文件 | 动作 | 必须通过的检查 |
| --- | --- | --- | --- |
| P0 | 本任务 research/ 的正式结果与下一阶段 PRD/design | 先逐项保存并审查本轮矩阵、primary、现有容量的原始失败收据，区分环境、计量、算法和未知原因；不操作 Trellis archive 或 commit；不得把历史/空窗差异直接归因到 SQL | 原 exit、样本数、身份、fixture、阈值逐项可追溯；所有 FAIL/UNVERIFIED/NOT_RUN 保持 |
| P1 | `residential-monitor/src-tauri/src/c3/raw_fold.rs` 的默认忽略 probe；必要时 `c3/service.rs` 的现有阶段 probe | 设计只读诊断，把扫描行读取成本与折叠开销分开；使用相同 SQL、同 reader snapshot、固定窗口和数值校验。诊断结果单列，不作为正式报告或性能 PASS | 精确 row count/字节校验、非空输出；无 SQL/schema/3120 阈值变化；保留缓存条件；停止计时前后对应同一 workload；probe 默认 ignored |
| P1（依赖诊断） | `c3/raw_fold.rs` | 只有 group/exit 哈希被量化为主要成本时，设计按实际分组基数建立索引、减少逐分钟重复 HashMap 查找的候选。不得按 session_pk 最大值分配；不得提前按 top_n 丢组 | 全 grouping/filter、缺失/空串/悬空ID、network字面unknown、规则fallback、重复chain、零字节、负分钟、排序tie、distinct session/minute、previous-window、跨窗 oracle 全部一致；溢出/取消/整报告deadline不变 |
| P1（依赖候选审查） | research/ 的独立构建、阶段与容量驱动 | 固定源码/工具链/锁文件/双边仪表；有限配对先决定是否保留候选。若需补充 A1000 短时语料，先明确诊断目的并通过强模型阶段审查，不能替代完整30天 | 每条退出码和实际样本数；分离 projection/scan/整报告；资源代价实测；无稳定收益则保留收据并回退 |
| P1（依赖阶段前置审查） | research/ 的完整 A1000 生成及21次查询收据 | 阶段方案获审查支持后，重新核对隔离路径与磁盘空间，生成同seed/start的完整30天A1000；不清理磁盘、不写真实库 | 43,200,000 minute、8,640,000 session、25,920,000 chain、2,592,000 receipt，marker/schema5/layout3/WAL/quick_check/oracle；21次host/top20全exit0且wall≤10000ms；首进程与物理冷页分开 |
| P2 | `bench/facade.rs`、`bench/process.rs` 或经批准的独立计量器；本任务 research/ | AC7全部应用文件写入仍需独立归属设计，纳入临时spool和日志，明确测量起止及遗漏。任何系统级追踪需独立批准，当前方案不启动trace | 原3轮300秒参数不变；同一仪表双边；明确未覆盖/丢事件/失败写边界；null不可换0，SQLite子集不可替代全部文件 |
| P2 | 经批准的安装态验收任务 | AC8 native+WebView、真实worker、隐藏恢复、24h soak与10k/1Hz/30分钟独立计划 | 每一环境有本轮身份与真实运行收据；合成回放不替代；未跑继续UNVERIFIED |
| P2 | `.trellis/spec/residential-monitor/storage/sqlite-contract.md`、相关backend规范、项目说明或业务skill | 范围内方案经强模型审查并完成检查后回写具体合同、适用工具和残留门；新增范围先取得对应批准 | `just ci`；受影响文档的独立`just docs-build`；secret/agent/task检查由主会话按涉及范围执行；独立强模型审查 |

## 禁止改变的验收边界

不放宽 10 秒 deadline、110% 比值门、AC7 CPU/文件写入阈值、完整语料或 21 次要求；不启用 AUTO_DELETE_ENABLED；不变 schema、核算、保留期、WAL/FULL、调度器；不把本轮 source/exe 与历史、安装态或未来候选混用。
