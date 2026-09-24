# raw_fold 隔离容量验证 — 2026-09-24

只操作 `C:/Users/lyh/AppData/Local/Temp/resiwatch-resource-measure-20260924-raw-fold`。未写安装库，未安装，未提交。生产 10 秒 deadline 未放宽。

身份见 `build-identity.json`。`monitor-db` SHA256 `9B34FFB5DE835A5A9E8DACCCB3D3E915C6C438BD8FAE9511BA641C7B99EA34BA`。生成参数与旧 layout3 相同：seed `20260919`，start `1787184000`，30 天。A50/A250 主库字节分别为 544940032 / 1697222656，与 layout3 行数规模一致。小时/日汇总表行数仍为 0。

CLI 是 `rank --by host --top 20` 与 `rank --by network --top 100`。`base_query` 自带家宽过滤。墙钟含进程启动；exit 7 是产品 deadline。

| 查询 | 结果 |
| --- | --- |
| A50 30 天 host，1 次 | exit 0，1559.8 ms |
| A250 30 天 host，8 次后按旧规则停止 | 6 次 exit 0，墙钟 9329.966–9894 ms；2 次 exit 7，10068.331 / 10069.851 ms |
| A250 1 天 host，1 次 | exit 0，3899.5 ms |
| A250 30 天 network，1 次 | exit 0，9963.8 ms；connectionCount 合计 1620000，等于 sessions×3/4 |

A250 host 30 天不是稳定通过。阶段探针 `a250-30d-stage.json` 与 CLI 冠军身份一致：会话投影 3883.478 ms（2,160,000 个会话），分钟扫描 6422.716 ms。两者之和已到 10 秒附近，coverage 和进程启动不再留余量。

未生成 A1000。A250 分钟扫描已占 6.4 秒，A1000 分钟行数是其 4 倍，全扫描不能落到 10 秒内。生成 6 GB 只为再记录一次超时没有新信息。下一步若继续优化，应先减少会话投影或分钟页读取，并用新身份重测 A250 host 30 天，不能沿用这份超时结果。

## 加速后复测

家宽过滤下推到投影，distinct 按会话只计一次。新 `monitor-db` SHA256 `2D77910A368C949C0E75C44CFFC2F828BA83A9A3E2972FC56EA8BFA1FBBB2D16`。同一库上：

| 查询 | 结果 |
| --- | --- |
| A250 30 天 host，21 次 | 全部 exit 0，最慢墙钟 9166.963 ms |
| A250 30 天 network，21 次 | 全部 exit 0，最慢 8636.415 ms |
| A50 30 天 host，21 次 | 全部 exit 0，最慢 1522.451 ms |

冠军身份与加速前一致。原始记录在 `fast-a250/`、`fast-a250-extra.json` 和 `repeat-rounds.json`。仍未生成 A1000：A250 已通过，但分钟行数 4 倍后全扫描仍预计超过 10 秒，这次没有新的阶段证据支持生成。
