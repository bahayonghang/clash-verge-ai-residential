# 收口 ResiWatch 失败的报告与矩阵门

## Goal

把前序任务留下的失败门收成可测修复，而不是再记一轮超时。用户要的是当前二进制上仍失败的报告与矩阵项被收口，且不放宽 10 秒 deadline 和 110% 比值门。

前序任务：`.trellis/tasks/archive/2026-09/09-19-resiwatch-resource-optimization`。本任务不重复已通过的 A50/A250 30 天 host/network 排名，除非修复后必须回归。

## Confirmed facts

- 2026-09-24 `monitor-db` SHA256 `2D77910A368C949C0E75C44CFFC2F828BA83A9A3E2972FC56EA8BFA1FBBB2D16`。同一 A250 隔离库 host 30 天 21 次全部 exit 0，最慢 9166.963 ms；network 21 次最慢 8636.415 ms；A50 host 21 次最慢 1522.451 ms。未生成 A1000。
- 2026-09-24 `monitor-bench` SHA256 `4C17FA3F7A8A37FD461387B7068F428031D7B2939718D0AEC8B5A45682878EC3`。冻结基线 exe `11C2542BC2C9F98A4B7517E67215E76884D8C01D25D64AE7FAF478571BF7633E` 已不在记录路径。14 份当前结果对照归档 baseline JSON，不是同窗口 AB。fixture hash 全部一致，全部守恒，无 1 秒 frame overrun。
- 主场景三轮相对归档 baseline：CPU 合计 15.40625→1.8125 秒，SQLite xWrite 70,595,256→32,718,048 B。每对写量 10,906,016 B，接近 2026-09-20 candidate 的 10,942,880 B。跨日 CPU 不能当成同窗口通过。
- 非空 1 分钟窗口：A250 30 天库，`default_auto_report_query`，`[1787184000, 1787184060)`。生产 `run_uncached` 7690.685 ms，tier Raw，250 连接。阶段：finalized days 0.076 ms，plan snapshot 0.021 ms，durable version 0.176 ms，raw query 6756.665 ms，reader close 0.961 ms。固定开销不是这 7.7 秒的来源。
- `default_auto_report_query` 在 `residential-monitor/src-tauri/src/c3/query.rs:672`，没有分类过滤，并带前一等长窗口比较。`load_sessions` 在 `raw_fold.rs:106`，`fill_raw` 在 `service.rs:811`。未过滤投影会读取窗口外会话；这是待证机制，不是已落地修复。
- 矩阵回放窗口没有已落盘 raw 行。首 reader 约 1.75–2.95 ms，比值大于 1.10 不能代表非空报告，也不能因此免除 110% 门。

## Requirements

### R1 非空分钟报告

默认自动报告在非空分钟窗口上，raw 查询不得再被全表会话投影主导。1 分钟 A250 首读必须保持与现有 250 连接、11725 download、3497 upload 等价，并记录 projection/scan 拆分。生产 10 秒 deadline 不放宽。

### R2 矩阵 110% 门

下列项相对同输入 baseline 的 ingest p95、首 reader p95 或 native private p95 大于 1.10，或 SQLite 写比值大于 1.10。当前数字来自跨日对照，不能直接当回归根因。修复或复测前必须先取得同窗口证据，或给出可复现的阶段归属。

| ID | 场景 | 失败指标 | 归档 baseline | 2026-09-20 candidate | 当前 |
| --- | --- | --- | --- | ---: | ---: | ---: |
| F1 | A1000 metadata | ingest p95 | 27.817 ms | 53.036 ms | 284.837 ms，max 863.497 ms |
| F2 | A250 backlog | ingest p95 | 9.059 ms | 15.321 ms | 96.392 ms，max 143.641 ms |
| F3 | A50 unchanged | ingest p95 | 9.853 ms | 8.878 ms | 73.236 ms，max 305.681 ms |
| F4 | A250 failed | ingest p95 | 13.820 ms | 14.821 ms | 23.148 ms，max 32.359 ms |
| F5 | A250 metadata | ingest p95 | 13.059 ms | 13.997 ms | 14.958 ms |
| F6 | A1000 metadata | native private p95 比 | 1 | — | 1.1275 |
| F7 | A1000 unchanged | native private p95 比 | 1 | — | 1.1193 |
| F8 | A50 metadata | SQLite xWrite 比 | 1 | — | 1.4235 |
| F9 | A250 failed | SQLite xWrite 比 | 1 | — | 1.2979 |
| F10 | A1000 metadata | SQLite xWrite 比 | 1 | — | 1.1787 |
| F11 | A250 backlog | CPU 比 | 1 | — | 1.1724 |
| F12 | 11 对矩阵 | 首 reader p95 比 | 1 | 约 0.91–1.24 | 1.3356–2.2240，绝对值 1.75–2.95 ms |

F12 保留为门，不把空 raw 窗口的毫秒级比值写成容量失败。

### R3 A1000 30 天报告

A1000 30 天 host 排名仍未在 10 秒内证明。没有新的阶段计划证明分钟扫描能落进 deadline 之前，不生成约 6 GB 语料只为再记一次超时。

### R4 同窗口主场景

主场景 CPU/SQLite 下降目前只对照归档 JSON。宣称 AC7 主场景通过之前，必须用仍可构建的 baseline 与当前 candidate 做同窗口三轮，或明确记录 baseline 不可重建及因此不能宣称通过。

## Acceptance criteria

| ID | 可观察结果 |
| --- | --- |
| AC1 | 同一 A250 库、同一 1 分钟窗口，默认报告连接数/上下行与 2026-09-24 探针一致；projection 与 scan 分开记录；首读 wall 低于当前 7690.685 ms，且不触达 10 秒 deadline。 |
| AC2 | A250 30 天 host 与 network 在修复后的新身份上仍 21 次 exit 0，最慢不超过 10 秒。 |
| AC3 | F1–F5 要么同窗口复测后 ingest p95 比 ≤ 1.10，要么有阶段证据证明不是产品回归且复测不再超过 1.10。不允许用跨日单次样本销项。 |
| AC4 | F6–F11 用同一规则收口。F12 只有在非空窗口首读也超过 1.10 时才改报告路径；空窗口比值单独保留，不单独触发 SQL 改动。 |
| AC5 | A1000 30 天 host 要么 21 次 exit 0，要么阶段计划证明当前算法不可能进 10 秒且该算法已被替换后再测。未测不记通过。 |
| AC6 | `AUTO_DELETE_ENABLED` 保持 false。不写安装库，不安装，不把 WebView、真实 worker 或 24 小时 soak 记成通过。 |

## Out of scope

- 安装、重启、真实库维护、WebView、真实 collector/后台 worker、窗口隐藏恢复、24 小时 soak。
- 打开自动删除。守恒门和容量门都通过之前不打开。
- 放宽 10 秒 deadline、110% 比值门或 1 秒 frame budget。
- 重做已通过且与本次修复无关的保留、VACUUM、字典老化证据。

## Notes

证据原文：`.trellis/tasks/archive/2026-09/09-19-resiwatch-resource-optimization/research/performance-20260924-current-binary/note.md` 与 `nonempty-1min-stages.json`。容量通过记录在同任务 `research/performance-20260924-raw-fold/capacity-note.md`。
