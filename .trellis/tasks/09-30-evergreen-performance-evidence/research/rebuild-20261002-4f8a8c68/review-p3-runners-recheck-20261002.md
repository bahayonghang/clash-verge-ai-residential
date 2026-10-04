# T05 P3 新 runner 复查（R1–R3 修复后）

日期：2026-10-02。方式：只读静态审查。没有执行 runner、cargo、native 程序、generator、p2-completion 或测试，没有打开 SQLite。逐项结果见同名 `.json`。首轮审查文件未修改。

## 结论

**P3_RUNNER_RECHECK_PASS**。R1–R3 已修复。门槛语义与原 runner 相同。没有发现新缺陷，无必须修复项。

## 身份

| 文件                              | SHA256                                                             |
| --------------------------------- | ------------------------------------------------------------------ |
| 原 `run-formal-replay.py`         | `6C88ACCA…81E9`（未变）                                            |
| 原 `run-retained-capacity.py`     | `9FA8C225…AAA4`（未变）                                            |
| `runners/run-formal-replay.py`    | `3009F02E3CEB42DFB1C01C0FB234F1BC677F345090D1D91347C96B5D408CEE90` |
| `runners/run-capacity.py`         | `AA33F884E0A464CEEBDF4338C55500B067B47301747039F04BBDAE79E1609FD1` |
| `runners/run-formal-replay.patch` | `111B4CA05C6679AA95F97E61FD47F3AA415B75929B46296D95AF12D62FCF4D90` |
| `runners/run-capacity.patch`      | `5AF47D8A8A49B95EF78B8F35138BF1586D430AD64B8F6D5C3871E0BEAE831B7C` |
| `p2-completion.py`                | `ECDEC6DF5821886A0C0DFC9016A4A86433A2F2B315E332C573F641A693DCB9FD` |

两个 v2 patch 的正文与原件对新副本重新计算的 `diff -u` 正文相同。v1 patch 的 hash 与首轮记录相同。

## 逐项结果

| 项                 | 结果 | 要点                                                                                                                                                                                                                                                                                                                                                                                       |
| ------------------ | ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| R1                 | PASS | capacity 先执行 `load_before = competing_load()`，再取 `started_utc`，随后紧接 `perf_counter` 与 Popen。generatedUtc 下界恢复原含义。                                                                                                                                                                                                                                                      |
| R2                 | PASS | replay 的 `INITIAL_LOAD` 断言位于 `assert not OUT.exists() and not DATA.exists()` 及 mkdir 之前。启动被阻断时不创建目录。                                                                                                                                                                                                                                                                  |
| R3                 | PASS | 两个 runner 的 `competing_load()` 相同。后缀匹配覆盖本批四个复制程序名；capacity 已含 monitor-db；其它 runner、builder、generator、oracle、p2-completion 的 python 进程可被匹配；本进程及其祖先（包装器、shell）被排除。扫描时 runner 自身没有存活的子进程。                                                                                                                               |
| 门槛语义           | PASS | 对 v1、v2 patch 的 +/- 行做多重集对比，差异仅为 `competing_load()` 函数体、R1 的 `load_before` 与 capacity receipt 行；R2 只改变行顺序。验证函数、case 与顺序、106 次、F1–F12、1.10/0.70/0.50、退出码与失败策略均未变。                                                                                                                                                                    |
| 旧树引用           | PASS | 旧路径、Temp、旧 marker/DB hash、run_root 均无命中。                                                                                                                                                                                                                                                                                                                                       |
| p2-completion 写入 | PASS | status 前缀为 `P2_GENERATION_ORACLE_PROBES_ENDED`。`corpus_identities["50"/"250"]` 的 db/marker/db_sha256/marker_sha256 与 generator 的 identity 文件相同，均为大写 64 位；路径与 capacity 由 asset_root 推导的路径相同。其它所读键未变。新 A250 的 host-30d、network-30d oracle 行与历史文件逐行相同，capacity 的 A250 30d 精确比对与新语料一致。bind.json 记录 DB hash 未变，-wal 为 0。 |
| fresh-only         | PASS | matrix、primary、capacity 及 data/matrix、data/primary 均不存在。                                                                                                                                                                                                                                                                                                                          |

## 风险记录（非阻断）

1. 主会话观测到 9 个 ferrots 进程。它们存在期间，两个 runner 会在启动断言处停止，且不写入任何输出。这是预期行为。
2. `p2-completion.py quiet-probes` 使用 generator 的 `competing_load()`。该函数只做精确名称匹配，不能检测正在运行的 replay/capacity 或其 `candidate-*.exe` 子进程。不要在 P4–P6 期间运行 quiet-probes。
3. quiet-probes 会改写 state.json，只改 stage_probes，保留 status 前缀。runner 只在启动时读取 state，并发改写最多导致启动前失败。
4. 若 quiet-probes 在 A250 库留下非空 -wal，capacity 会在 WAL 断言处停止。这是停止，不是误判通过。quiet-probes 只核对 DB hash，不核对 WAL 大小。
5. 祖先排除依据 psutil 父 PID。Windows 复用旧父 PID 时，复用该 PID 的进程会被跳过。概率低。
6. 脚本名按子串匹配，无关命令行含相同名称时会误停启动。这是误停，不是误判通过。
7. 首轮已记录、仍未改变的事项：execution-contract.json 未记录 state.json 与 P2 oracle summary 的 hash；candidate source label 为 `candidate-copy-after.json` 的 hash；runner 不核对 dist；capacity 结束前不要改写 status 前缀。

本轮没有在两个允许文件之外写入任何临时文件。
