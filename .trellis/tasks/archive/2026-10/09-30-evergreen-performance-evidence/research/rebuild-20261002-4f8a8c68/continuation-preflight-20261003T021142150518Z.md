# T05 恢复前置检查

日期：2026-10-02，America/Chicago；实际 UTC：`2026-10-03T02:11:42.150518+00:00`。

**BLOCKED_NOT_RUN**。身份/收据核验：**FAIL**。正式执行数：matrix 0/22、primary 0/6、capacity 0/106。

## 核验结果

已实测核对仓库与 candidate 各 108 项、baseline 103 项、三套各 36 项 dist、四个复制程序及对应 compiler-artifact 原程序、新旧 runner 与 P3 复查 hash、P2 六份 native 收据及原始/gzip/可读日志、fresh-only 与指定路径。失败检查：reviewed hash runners/run-formal-replay.v1.patch, reviewed hash runners/run-capacity.v1.patch。

A50/A250 完整 30 天生成与独立 SQL oracle 收据已绑定。marker hash、主库 byte size 和 WAL/SHM 大小为本轮实测。主库 hash 与 quick_check 引用原 P2 记录，本轮未重新哈希主库、未打开 SQLite。正式 capacity 启动前仍须由 runner 重检。

## 首失败

`generator-driver.receipt.json` 保留 STOPPED_FIRST_FAILURE / KeyError / `'upload'`，driver exit 为 1。A250 30 天生产诊断保留 `DeadlineExceeded("report query")`，production first read 为 10035.869999999999 ms，raw query 为 13016.4438 ms，native test exit 为 0。测试进程退出码不能覆盖生产错误。分钟诊断 PASS_DIAGNOSTIC；all/residential raw-fold 探针 NOT_RUN。原诊断有 ferrots 竞争负载，性能原因未查明。state 与首失败文件前后 hash 相同。

## 当前阻断

固定机制匹配到 6 个竞争进程；未终止任何用户进程。以下记录只含 PID、程序名、创建 UTC 与固定匹配原因。

| PID | 程序名 | 创建 UTC | 固定匹配原因 |
| --- | --- | --- | --- |
| 43996 | pwsh.exe | 2026-10-02T18:41:40.092951+00:00 | reviewed fixed command token: ferrots |
| 48028 | pwsh.exe | 2026-10-02T14:26:00.566294+00:00 | reviewed fixed command token: ferrots |
| 51228 | bash.exe | 2026-10-02T14:27:51.485206+00:00 | reviewed fixed command token: ferrots |
| 52584 | bash.exe | 2026-10-02T14:27:51.496589+00:00 | reviewed fixed command token: ferrots |
| 55612 | ferrots.exe | 2026-10-03T00:57:20.825126+00:00 | reviewed fixed command token: ferrots |
| 62832 | python.exe | 2026-10-02T18:41:41.757636+00:00 | reviewed fixed command token: ferrots |

快照不可保证所有无关负载已结束。不可读取或扫描时退出数量：179。本轮派发明确排除正式执行及 quiet-probes；主会话应在竞争负载结束并重新核对前置后发出明确 GO。

## 待执行命令

以下命令均未执行，按 matrix→primary→capacity 串行推进，每阶段先审查实际收据。

- matrix，22 次：`python -B "D:\Documents\Code\Github\clash-verge-ai-residential\.trellis\tasks\09-30-evergreen-performance-evidence\research\rebuild-20261002-4f8a8c68\runners\run-formal-replay.py" --set matrix`
- primary，6 次：`python -B "D:\Documents\Code\Github\clash-verge-ai-residential\.trellis\tasks\09-30-evergreen-performance-evidence\research\rebuild-20261002-4f8a8c68\runners\run-formal-replay.py" --set primary`
- capacity，106 次：`python -B "D:\Documents\Code\Github\clash-verge-ai-residential\.trellis\tasks\09-30-evergreen-performance-evidence\research\rebuild-20261002-4f8a8c68\runners\run-capacity.py"`

保持 10 秒、110%、0.70、0.50 门及原 106 次容量计数。普通性能 FAIL 与全部应用文件归属 UNVERIFIED 不新增为容量启动门。quiet-probes 不得与 P4–P6 并行。缺少阶段证据不勾选原 AC。

A1000 完整 30 天、F12 非空同窗口、全部应用文件归属、物理冷缓存、安装态/WebView/worker/24h/peak、hosted，以及 P1 缓存键正确性/实际 linker/SDK 保持未验证。

报告生成工具首失败：读取时未指定 UTF-8 的 UnicodeDecodeError（exit 1）；首次报告构造因 Python 缺少 tzdata 产生 ZoneInfoNotFoundError（exit 1，未写 artifact）。后续使用 Windows TimeZoneInfo 已观测的 -05:00 偏移，无依赖安装。两项工具失败独立于 P2 首失败和正式结果。

JSON SHA256：`7AA781157B3BCD7996CE27713871498160A85ED0C89F3C96CBA43A6BE0DE70C4`。本轮仅新增本 Markdown 与同名 JSON。实际工具外层退出由主会话按完成结果独立记录。

**STOP_WRITING**。
