# T05 缓存最小回退与本地验证

日期：2026-10-01（America/Chicago）。适用工具：Claude Code、Codex、Grok Build、Kimi Code、OMP。状态：PASS_LOCAL_PRODUCT_CHECKS；T05 性能/容量验收仍未完成。

仅恢复 raw_fold.rs 的 Dict、load_sessions、identity_from_dict/rule_name/load_dict 三个生产片段。生产部分与已审 raw_fold-contract-restored.rs 一致；测试区逐字保持。kind == "process" 的缺失过滤合同修复保留。SQL、schema、3120 分钟阈值、deadline、取消、维护和删除范围未修改。回退相对 retained 源为 45 行新增、69 行删除；相对 HEAD 的生产差异只保留该 process 限定修复。

| 检查 | exit | 实际结果 |
| --- | ---: | --- |
| Rust fmt --check | 0 | 通过 |
| raw_fold targeted | 0 | 7 passed，0 failed，1 ignored |
| service targeted | 0 | 53 passed，0 failed，3 ignored |
| clippy workspace/all-targets -D warnings | 0 | 通过 |
| 完整 just ci | 0 | 前端 300；Rust 单元 546，集成 3；root Node 203；0 失败 |
| git diff --check | 0 | 通过 |

完整 just ci 包含 monitor 的版本/安装锁文件/icons/typecheck/lint/前端测试/build、Rust fmt/clippy/workspace 测试、安全扫描，以及根 npm run ci。忽略的 6 个 Rust 探针保持未运行；普通测试不能代替 30 天容量、正式性能、安装态或 hosted 门。

最终源码 manifest 为 8EE21DEACDFE15166BC97A92D78E24445EF26AC90D4813FB30EB368C180996CA（108 项），检查前后相同。raw_fold.rs SHA256 为 4D6E8FEB88B142E09F8E62D60BAF20BAC970F11CB8D8E9108B08CFFBF717E6AA。各项原生检查 receipt 绑定该 manifest，gzip stdout/stderr 的 hash 均核对通过。两份 debug test exe 的身份见 validation-summary.json；这些 exe 不构成 release 或性能身份。

旧 matrix/中断 primary/完整恢复 primary 的 195 个证据文件 hash 保持。不得把旧 retained exe 的性能结果迁移给本次回退源码。容量仍为 BLOCKED、0/106；此前缺失资产 preflight 保留，未恢复或重建资产。docs 与最终独立审查由主会话安排。

证据：本目录 raw_fold-before.rs、raw_fold-rollback.patch、rollback-application.json、source-manifest-before/after-checks.json、五项 receipt/raw 日志、checks-summary.json、validation-summary.json。
