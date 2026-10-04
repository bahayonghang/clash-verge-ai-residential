# T05 Primary 完整恢复结果

日期：2026-10-01。适用工具：Claude Code、Codex、Grok Build、Kimi Code、OMP。状态：UNVERIFIED；等待独立检查。

新批次完整执行三对六次原生调用。每次独立空目录，30 秒预热、300 秒实际测量；baseline → candidate。原中断批次未拼接，原证据 23 个文件 hash 全部保持。

| native | exit | wall 秒 | CPU 秒 | SQLite xWrite B |
| --- | ---: | ---: | ---: | ---: |
| primary-r1-baseline | 0 | 343.018343 | 7.078125 | 23531752 |
| primary-r1-candidate | 0 | 342.462933 | 0.0 | 10906016 |
| primary-r2-baseline | 0 | 342.566827 | 9.40625 | 23531752 |
| primary-r2-candidate | 0 | 342.111865 | 2.515625 | 10906016 |
| primary-r3-baseline | 0 | 343.331350 | 7.328125 | 23531752 |
| primary-r3-candidate | 0 | 340.980905 | 0.3125 | 10906016 |

六次 validation_errors 均为空；三个 pair 的 fixture、平台、时区相同；流量守恒，原 writer 初末 WAL/FULL 均通过。raw gzip 解压后的 stdout/stderr hash 与各收据相同。两侧源文件 103/108 项和四个 exe 在完成后仍匹配冻结身份。

| 指标 | baseline 合计 | candidate 合计 | 比值 | runner 状态 |
| --- | ---: | ---: | ---: | --- |
| AC7 CPU | 23.8125 | 2.828125 | 0.11876640419947507 | PASS |
| AC7 SQLite subset only | 70595256 | 32718048 | 0.46345958430974454 | PASS |
| AC7 all application files | None | None | None | UNVERIFIED |

第 1 轮 candidate 的 native_before.cpu_seconds 与 native_after.cpu_seconds 均为 9.765625 秒，报告差值 native_cpu_seconds 为 0.0 秒。原因未查明，不能推出零计算。正式统计仍使用已批准三轮合计。全部应用文件写入为 null，因此 AC5 整体保持 UNVERIFIED。

driver 实采 exit=3，wrapper 记录 exit=3，PowerShell 预定 exit=3，执行工具实采外层 exit=1。原因未查明。wrapper stderr 文件为空；独立保留实际码，不用 driver 替代。外层 wall 为 2054.7138146 秒。

证据：`formal-primary-recovery-20261001/summary.json`、`formal-primary-recovery-20261001.completion.json`、`.driver.json`、`.outer.json`、`.preflight-final.json` 和 `.planned-execution-contract.json`。原失败 preflight 与原审查转写错误均保留；正确参数/身份见恢复计划的启动前绑定表。

检查：六次正式 native、源文件/exe/runner/wrapper 身份、argv/日志 hash/原证据完整性、`git diff --check` 退出 0。未运行 build、产品测试、完整 just ci 或容量测量。没有改产品文件或共享 spec。

容量 106 次：NOT_RUN。完成本批后停止，等待独立 primary 检查。旧 retained exe 的结果不适用于任何后续回退源码或新身份；安装态、WebView、真实后台 worker、24h、peak 与物理冷缓存保持未验证。
