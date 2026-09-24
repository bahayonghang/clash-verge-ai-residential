# 当前二进制复测 — 2026-09-24

`monitor-bench` SHA256 `4C17FA3F7A8A37FD461387B7068F428031D7B2939718D0AEC8B5A45682878EC3`。冻结基线 exe `11C2542BC2C9F98A4B7517E67215E76884D8C01D25D64AE7FAF478571BF7633E` 已不在记录路径，本轮没有重建它。比值对照 `performance-20260920-aux-series/` 的 baseline JSON，不是同窗口 AB。14 份当前结果的 fixture hash 都与对应旧 baseline 一致，全部守恒，均无 1 秒 frame overrun。

## 主场景

A250、1Hz、counters、archive complete、30 秒预热 + 300 秒实测、`query_every_frames=0`。三对均 300 次提交。

| 项 | 归档 baseline 合计 | 当前 | 相对归档 baseline |
| --- | ---: | ---: | ---: |
| OS CPU 秒 | 15.40625 | 1.8125 | 下降 88.2353% |
| SQLite xWrite 字节 | 70,595,256 | 32,718,048 | 下降 53.6540% |

每对当前 SQLite 写都是 10,906,016 B。2026-09-20 candidate 是 10,942,880 B。写量接近，不能把跨日 CPU 当成新的同窗口通过证明。SQLite 子集仍不是全部应用文件写入。

## 矩阵

参数与 2026-09-20 相同：5 秒预热 + 30 秒，`query_every_frames=5`，metadata 变化 100%。110% 门仍未通过。相对旧 candidate，下面几项 ingest p95 明显变差，不能归因到单样本：

| 场景 | 归档 baseline p95 | 2026-09-20 candidate p95 | 当前 p95 | 当前 max |
| --- | ---: | ---: | ---: | ---: |
| A1000 metadata | 27.817 ms | 53.036 ms | 284.837 ms | 863.497 ms |
| A250 backlog | 9.059 ms | 15.321 ms | 96.392 ms | 143.641 ms |
| A50 unchanged | 9.853 ms | 8.878 ms | 73.236 ms | 305.681 ms |
| A250 failed | 13.820 ms | 14.821 ms | 23.148 ms | 32.359 ms |

首 reader 报告 p95 约 1.75–2.95 ms。回放窗口没有已落盘 raw 行，这个比值不代表非空报告。

## 非空分钟窗口

同一 A250 30 天库，`default_auto_report_query`，区间 `[1787184000, 1787184060)`，模拟 now 等于区间终点，保留 30 天。生产 `run_uncached` 7690.685 ms，exit ok，tier Raw，250 连接。第二次连接的阶段拆分：

| 阶段 | 毫秒 |
| --- | ---: |
| finalized days | 0.076 |
| plan snapshot | 0.021 |
| durable version | 0.176 |
| raw query | 6756.665 |
| reader close | 0.961 |

plan snapshot 内部还会再查一次 finalized days，不能把两次相加当生产路径。未驱逐 OS page cache。固定开销不是这 7.7 秒的来源；成本在 raw 查询。原始 JSON：`nonempty-1min-stages.json`。
