# 失败门跟进测量 — 2026-09-24

只读 `C:/Users/lyh/AppData/Local/Temp/resiwatch-resource-measure-20260924-raw-fold/corpus-a250-30d/monitor.sqlite3`。未写安装库，未安装，未打开自动删除，未放宽 10 秒 deadline。OS page cache 未驱逐。

## 非空 1 分钟

窗口 `[1787184000, 1787184060)`。修复前未过滤投影 9972.096 ms、2,160,000 个会话；当前分钟扫描 16.646 ms；前一分钟扫描 15.788 ms。连接 250，upload 3497，download 11725。

同一库上，短窗口投影改为分钟主键上的 distinct `session_pk` 再按主键取会话。修复后投影 1.151 ms、250 个会话；当前扫描 0.274 ms；前一分钟扫描 0.009 ms。连接、上下行和榜首 identity 与修复前相同。前一分钟仍是 0，因为语料从 `1787184000` 开始。

生产 `run_uncached` 首读 7.271 ms，exit ok，tier Raw，连接 250，upload 3497，download 11725。低于修复前的 7690.685 ms，未触达 10 秒 deadline。原始文件：`unfiltered-1min-split-before.json`、`unfiltered-1min-split-after.json`、`nonempty-1min-after.json`。

投影跨度含已保留的前一等长窗口。跨度不超过 3120 分钟（两次 26 小时）才走窗口投影。更长区间仍全表投影。

## A250 30 天

`monitor-db` SHA256 `D0DFEF0572F1AE72934F109D0EE9E52ADC6779BC59FF2D2119DFD059B133793E`。区间 `[1787184000, 1789776000)`，`--tz UTC`，家宽过滤。30 天跨度 43200 分钟，不走窗口投影。

页缓存未热时，一次 host rank exit 7，墙钟 10121.934 ms，stderr `report query`。随后一次无 deadline 的阶段拆分把分钟页读进缓存：投影 2901.177 ms、1,620,000 会话，扫描 4224.710 ms，download 380699625，与归档 `a250-30d-stage.json` 的 download 相同。紧接着的 host rank exit 0，墙钟 7137.500 ms。

同一热缓存上各 21 次，全部 exit 0，tier raw，榜首 identity 不变：

| 排名 | 次数 | 最慢 | 最快 |
| --- | ---: | ---: | ---: |
| host top 20 | 21 | 8029.563 ms | 6470.436 ms |
| network top 100 | 21 | 7177.571 ms | 5773.372 ms |

索引：`a250-30d-rank-index.jsonl`。阶段：`a250-30d-residential-split-warm.json`。

## A1000

未生成。热缓存上 A250 30 天分钟扫描 4224.710 ms，A1000 分钟行数是其 4 倍。窗口投影不覆盖 30 天全范围。按这次扫描外推，A1000 全扫描不能进 10 秒。未测，不计通过。

## 矩阵与主场景

记录路径上的 baseline `monitor-bench-baseline-v2.exe`（SHA256 `11C2542BC2C9F98A4B7517E67215E76884D8C01D25D64AE7FAF478571BF7633E`）仍不存在。本轮没有同窗口 AB，不宣称 F1–F11 或主场景 CPU/SQLite 通过。F12 的空窗口比值没有单独触发这次 SQL 改动；触发证据是上面的非空投影耗时。
