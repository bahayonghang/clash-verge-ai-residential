# T05 P3 新 runner 绑定静态复审

日期：2026-10-02。方式：只读静态审查。没有执行 runner、cargo、monitor-bench、monitor-db、generator 或测试，没有打开 bench-data 下的 SQLite。逐项结果见同名 `.json`。

## 结论

**P3_RUNNER_REVIEW_FAIL**，首个失败项为第 2 项（轻微）。第 6 项同为 FAIL。需要 R1–R3 三项修复后重新审查。门槛、case、指标和退出码语义本身没有发现改动。

## 逐项结果

| #   | 项目                       | 结果         | 要点                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| --- | -------------------------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | 原 runner 未变             | PASS         | replay `6C88ACCA…`、capacity `9FA8C225…` 与合同一致；`run-live-recorded.py` 原件与新副本同为 `88550BB4…`。两个 .patch 正文与重新计算的 `diff -u` 正文 SHA256 相同。                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| 2   | 差异仅限绑定/收据/竞争负载 | FAIL（轻微） | 验证函数、expected options、matrix 11 case 及顺序、primary r1..r3、capacity 21/21/1/21/21/21=106、F1–F12、1.10/0.70/0.50、退出码 2/3/0 与失败策略均未改。偏差：`run-capacity.py:148` 的 receipt 先取 `started_utc`，再执行 psutil 扫描，然后才 Popen。`started_utc` 是 `:178` generatedUtc 校验的下界，下界因此提前一个扫描时长。原 `:129` 在两者之间没有其它操作。wall_ms 不受影响。                                                                                                                                                                                                                                                                   |
| 3   | 无旧树引用                 | PASS         | 检索 build-20260930、candidate-retained-20260930、retained-capacity-20260930、AppData、Temp、AA4A7ABD、667A21E9、e6fdd679、run_root 等均无命中。capacity 仍按原机制只读 `../corpus-a250-verification.json` 取 SQL 与 A250 30d 期望行。                                                                                                                                                                                                                                                                                                                                                                                                                  |
| 4   | state 键与 hash 大小写     | PASS         | 所用键在 state.json 中均存在；`corpus_identities` 由 `generator.py:558-582` 以 `"50"`/`"250"` 写入 db_sha256/marker_sha256（大写）。exe 与 108/103 项 manifest hash 均为大写，runner 对计算值调用 `.upper()` 后比较。`candidate-copy-after.json` 等于 `state.source_manifest`；`baseline-instrumented-source.json` 等于 `state.baseline_manifest`（五项接入 hash 与合同一致）。                                                                                                                                                                                                                                                                         |
| 5   | 输出 fresh-only            | PASS         | OUT 为 `<EvidenceRoot>/matrix                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | primary | capacity`，DATA 为 `<AssetRoot>/data/matrix | primary`，均先断言不存在。这些目录当前不存在。没有写入旧证据的路径。 |
| 6   | 竞争负载启动断言           | FAIL         | 门槛计算未变，记录字段不进入 failures/gates/exit。psutil：replay 顶层导入，capacity 函数内导入；Python314 可导入 psutil 7.2.2。缺陷 a：`run-formal-replay.py:38-39` 先创建 OUT/DATA，`:57-58` 才断言；启动被阻断后留下空目录，固定路径不能重试，且不得删除。capacity 的顺序正确。缺陷 b：名称只匹配 `monitor-bench.exe`/`monitor-db.exe`/`cargo.exe`/`rustc.exe`/`ferrots`；本批复制程序名为 `baseline-monitor-bench.exe`、`candidate-monitor-bench.exe`、`candidate-monitor-db.exe`、`candidate-library-tests.exe`，不会命中；Python 驱动也不会命中；capacity 另缺 `monitor-db.exe`。P4/P5/P6 互相重叠时检测不到，`competing_load_observed` 保持为空。 |
| 7   | 风险                       | NOTED        | 见下节。                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |

## 必须修复

- **R1**：`run-capacity.py` 中先调用 `competing_load()`，再生成含 `started_utc` 的 receipt，使 generatedUtc 下界与原 runner 一样紧邻 Popen。
- **R2**：`run-formal-replay.py` 将 `INITIAL_LOAD` 检查和断言移到 `assert not OUT.exists() and not DATA.exists()` 及 mkdir 之前。
- **R3**：两个 runner 的 `competing_load()` 增加本批四个复制程序名（或匹配 `STATE["executable_identities"]` 路径）；capacity 增加 `monitor-db.exe`；匹配 `run-formal-replay.py`、`run-capacity.py`、`builder.py`、`generator.py`、`read-corpus-oracle.py` 的 Python 命令行，并排除 `os.getpid()`。

修复后重新生成两个 .patch，记录新 runner SHA256，再复审。

## 风险记录

1. candidate source label 改为 `manifest-sha256:` 加 `inputs/candidate-copy-after.json` 的小写 SHA256。argv 与 expected 使用同一变量，验证一致；标签与历史批次不同。
2. candidate 源码从隔离副本 `candidate_source` 核对，不再核对仓库工作树。这与所建 exe 绑定一致；P1 之后的仓库漂移不在检查范围。两个 runner 都不核对 dist_manifest（原 runner 同样不核对）。
3. 两个 runner 断言 `state.status` 以 `P2_GENERATION_ORACLE_PROBES_ENDED` 开头。主会话若在 P3–P5 之间改写 status，后续 runner 会停止。需在 capacity 结束前保持该 status，或重新绑定。
4. stage probe 结果为 `FAIL_DIAGNOSTIC_RETAINED` 时 generator 也写入同一 status 前缀；runner 不读取 stage_probes。这与合同中诊断与正式门分别记账一致。
5. runner 读取 state.json 时不固定其 hash；execution-contract.json 记录 state_status，不记录 state.json SHA256 或 P2 oracle summary hash。建议补记。
6. capacity 以 utf-8-sig 读取 `corpus-a250-verification.json`。该文件 SQL 含 UTF-8 中文字符。本机 locale 为 cp936，原 `read_text()` 在位置 3177（字节 0xAE）抛出 UnicodeDecodeError。新读法使 SQL 文本与文件字节一致，SQL 语义不变。
7. capacity 的 A250 30d 期望行仍来自历史 oracle 文件；A50 与 1d 只有运行时 SQL 结果；runner 不交叉核对 `oracle/run-20261002` 的 P2 输出。
8. `run-capacity.py` 重复 `import sys`，无影响。

## 审查偏差

审查中有两个临时 diff 输出（d1、d2）写入 `C:/Users/lyh/AppData/Local/Temp`。它们在仓库之外。本角色禁止删除，因此未删除。
