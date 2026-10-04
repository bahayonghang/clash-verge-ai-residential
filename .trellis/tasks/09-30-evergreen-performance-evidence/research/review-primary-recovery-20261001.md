# Primary 恢复批次独立审查（2026-10-01）

结论：完整执行证据通过静态核对，已满足已批准 106 次容量测量的顺序前置。容量执行仍由主会话安排，等待当前有限准备检查结束。T05 整体验收保持 UNVERIFIED，已有正式 matrix FAIL 保留。

## 核对范围与结果

仅审查 `formal-primary-recovery-20261001/`、最终 preflight、driver、outer、completion、结果说明与 runner/wrapper 源码；没有运行 benchmark、构建、测试或数据库读取。独立核对结果见 `review-primary-recovery-20261001.validation.json`。

六次 native 均为 finished、exit=0、validation_errors=[]，三轮按 baseline → candidate 串行完成。预定 argv 与实际 receipt 完全一致，各目录独立，无旧批次拼接。固定参数为 A250、1 Hz、counters、complete、metadata 100、30 秒预热、300 秒实际测量、query 0、seed 20260919、start 1800001800、virtual_time=false。六次实际测量 wall 均略大于 300 秒。

六份原始 stdout JSON、保存 JSON、results 副本相等。12 份 stdout/stderr gzip 解压 hash 与 receipt 相等，规范化文本日志与原始数据相符。参数 fixture 按 facade.rs 的版本 1 字段独立重算为 `e4f93c4c583bec372f8e675e66e5004823987f6832bf1387c0a4f54e965004f1`；六次均匹配，三对平台与时区匹配。

每次 frames/commits/native samples 为 300，采样帧连续 1..300，ingest/archive/whole_tick 计数为 300，reader/live_query 计数为 0。流量初值加增量等于终值，conserved=true。初末 writer 为 WAL、synchronous=2（FULL）；archive_ok=1116、failed=0，SQLite xWrite failed_calls=0。query 0 没有 reader 性能验收含义。

baseline 自报 exe hash 为 `06BAC94FD88E37DCD8971893BF79C91A05E9775CEED4A06AB7E2F10B690F3DD9`，candidate 为 `9604FBE6ED24C6DC4E40E556A6AD7F3338477582B223E15D20B795DDF73A7A5E`，与冻结合同相等。source_revision 分别为 c278bb7b56603001e32e353d2ee589dccef0bfe9 与 manifest-sha256:860165f3183ac8bfaac2d74af6f29e793575a91a82b1bb23d86ee975be5c0462。owner completion 记录源文件 103/108 项、四个 exe 完成后匹配；本次只审查收据，没有再次广泛读取源码或 exe。runner、live wrapper、outer script 的窄 hash 独立核对匹配。原中断批次 23 个研究证据文件独立 hash 核对全部保持。

## 三轮合计

| 指标 | baseline | candidate | 比值 | runner 结论 |
| --- | ---: | ---: | ---: | --- |
| CPU 秒 | 23.8125 | 2.828125 | 0.11876640419947507 | PASS（阈值 0.70） |
| SQLite xWrite 字节子集 | 70595256 | 32718048 | 0.46345958430974454 | PASS（阈值 0.50） |
| 全部应用文件写入 | null | null | null | UNVERIFIED |

三轮 CPU 差值 baseline 为 7.078125、9.40625、7.328125；candidate 为 0.0、2.515625、0.3125。SQLite 每轮为 23531752 → 10906016。独立重算总和、比值和门槛状态均与 summary 相等。SQLite 数字只覆盖成功 SQLite xWrite 请求字节；spool/allapplication null 不按零处理。

## 计量异常与退出码

首轮 candidate 原 native_before.cpu_seconds=9.765625、native_after.cpu_seconds=9.765625，reported delta=0.0；300 个中间 CPU 样本也全部为 9.765625。该轮存在 300 次 ingest，p50=5.1691 ms，SQLite xWrite=10906016 字节。CPU 计量异常原因未查明，不能推出零计算，也不能归因于计数分辨率。Windows 源 process.rs 使用当前进程 GetProcessTimes 的 kernel+user FILETIME 换算，facade.rs 使用前后差值；现有 JSON 没有原 FILETIME/错误码来解释该异常。按已批准协议保留原值和三轮合计的 runner PASS；CPU 成效解释必须同时披露异常，不将 runner 结果扩展为原因结论。owner 的初始分辨率归因已更正，历史 native 证据没有重写。

runner 源最后返回规则为：失败→2；任何 UNVERIFIED→3；其余→0。本次 execution_failures=[]、无 FAIL、全应用写入 UNVERIFIED，因此 driver exit=3 与该规则一致；wrapper process.wait() 记录 3。outer receipt 记录预定 3、实际执行工具 1，来源为 functions.exec → tools.write_stdin/session 58219/chunk 69c01d。实际 outer=1 与 driver=3 的差异原因未查明，分别保留，未将外层错误码转换为 PASS。六份 finished native 收据、完整结果与最终 driver 收据共同证明本批实际执行完成。

## 验收边界与下一步

缺测 allapplication 不增加为已批准 capacity 顺序前置的新门。106 容量仍 NOT_RUN；允许在当前有限检查结束后按已批准固定容量合同执行。正式 matrix F2/F6-F10/F12 FAIL、非空 reader 独立结果、A1000 正式验收、全部应用文件写入、安装态 WebView/worker/24h/peak 与物理冷缓存仍按各自原状态保留。本次不改变门槛、不重跑测量、不判定 cache 回归因果，也不将冻结候选作为交付 PASS 或替换原 baseline。后续源码回退与新 exe 无法继承本批身份结果。

验证状态：静态 JSON/收据/hash/算术 PASS；Lint/TypeCheck/产品测试未运行，当前仅为证据审查，不伪填完整产品 PASS。
