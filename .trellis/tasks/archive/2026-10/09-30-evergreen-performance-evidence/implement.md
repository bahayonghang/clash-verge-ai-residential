# T05 已批准执行清单

用户于 2026-09-30 批准项目内实施；子任务按执行顺序启动。

## Order

1. 读取原始 AC7/AC8 和 benchmark-harness，冻结 source、exe、fixture、seed、窗口、时区、机器负载。已确认旧 baseline exe 路径当前不存在；A250 隔离库路径存在，但本轮未打开数据库。
2. 从源码重建 baseline 和 candidate，分别复制到隔离目录并 hash；不覆盖安装态。
3. 运行阶段探针。RESIWATCH_RAW_FOLD_STAGE_DB/OUT、RESIWATCH_RAW_FOLD_START/END/FILTER 必须指向生成隔离库；分别记录 all/residential。
4. 阶段证据支持后提交最小算法设计；在批准范围内实施并加精确 oracle。无证据不改调度器。
5. 严格复跑既定同窗口矩阵、AC7 实时三轮和完整容量；不能用 virtual-time 或小库替代。
6. 回写 PASS/FAIL/UNVERIFIED 和边界；完整门未过时任务不记 completed。

## Required Checks

- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib raw_fold::`
- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c3::service::`
- `cargo test --release --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c3::raw_fold::tests::isolated_raw_fold_stage_proof -- --ignored --exact --nocapture（先设置隔离库环境）`
- `baseline/candidate: replay-facade --active 250 --hz 1 --duration-secs 300 --warmup-secs 30 --workload counters --archive complete --source-revision <对应源码> --dir <各轮新隔离目录>（3轮，不用 --virtual-time）`
- `完整 30 天 A50/A250/A1000 和 F1–F12 参数沿用原 benchmark-harness，逐次保留原始退出码`
- `just ci`
- `git diff --check`

新脚本、recipe 与测试命令是获批后的交付，不表示当前已存在或已通过。命令的预期失败、环境阻断和正式验收必须分别记录。仅最后一条 native 命令成功不能覆盖前面的失败。

## Review

强模型逐项核对 PRD、diff、检查证据和持久回写。共享文件按照父任务顺序串行处理；先读取前序改动，不得覆盖其他任务工作。

## Closure

全部 AC 有证据后才能声明本任务完成；缺少正式或动态证据保持未完成。提交、归档另需用户授权。
