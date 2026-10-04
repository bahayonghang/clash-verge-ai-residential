# T05 保留实现与正式输入

日期：2026-09-30。适用工具：Claude Code、Codex、Grok Build、Kimi Code、OMP。强模型主会话批准范围，独立强模型审查身份、语义和门；低成本模型未决定算法或放宽验收。

## 保留的产品改动

- `residential-monitor/src-tauri/src/c3/raw_fold.rs`：将 `__unknown__` 缺失过滤特例限定为 process，恢复 network 的字典值等值合同；字典 identity 按实际引用 lazy intern，避免重复字符串哈希和未引用文本进入最终 pool；chain 派生结果按完整原始 pool ID 复用。rule fallback 仍逐 session 求值，不缓存最终 rule。空串、NULL、悬空ID、raw chain 与排序 tie 保持原定义。
- 同文件保留语义、资源及独立 MinuteSet 集合/累加测试。连续分钟 `last` 缓存已回退，最终 MinuteSet 仍为 96 B。详见 `minute-set-results.md`；退出的 `candidate-final-20260930` 收据与exe保持。
- `residential-monitor/src-tauri/src/bench/facade.rs`：现有 inventory 在原 writer 连接读取实际 journal_mode 和 synchronous；初值在计量前，末值在 wall/CPU/VFS 采样结束后。top synchronous 标签从末值映射。读操作不改 PRAGMA 设置和测量窗口。相同 instrumentation 同步隔离 baseline。

无 SQL、schema、3120 分钟阈值、核算、10 秒 deadline、取消、AUTO_DELETE_ENABLED、WAL/FULL 配置或调度器改动。无安装、真实库、控制器、凭据、系统trace、提交、归档或push动作。

## 最终保留身份

目录：`candidate-retained-20260930/`。双方 source-before/source-after 集合和值相同；reviewer另核对当前完整源、103/108项清单、4exe hash/大小/manifest绑定。独立结果：`review-retained-candidate-validation.json`，errors=[]。

| 可执行文件 | SHA256 |
| --- | --- |
| baseline-monitor-bench.exe | `06BAC94FD88E37DCD8971893BF79C91A05E9775CEED4A06AB7E2F10B690F3DD9` |
| candidate-monitor-bench.exe | `9604FBE6ED24C6DC4E40E556A6AD7F3338477582B223E15D20B795DDF73A7A5E` |
| candidate-monitor-db.exe | `8EE782EDE8F6F61997CCADC7476B48963A33410C36EB25DF2F03E4FB18F6BA13` |
| candidate-library-tests.exe | `74ECF63AC02659AFF87AC5B6E2AE1264552E57630441E9337DB65450629BD638` |

candidate manifest SHA256：`860165F3183AC8BFAAC2D74AF6F29E793575A91A82B1BB23D86EE975BE5C0462`。baseline manifest：`1120B9D2B5C457F856BCD5F51ED9C88E0612E978AB4C7B76B4816640E1C6678F`。

baseline revision `c278bb7b56603001e32e353d2ee589dccef0bfe9`；保留 schema4。candidate 保留 schema5/layout3及上列批准改动。相同计量补丁不改变 baseline 的原始生产算法。工具链 Rust/Cargo1.98.0，release、locked、x86_64-pc-windows-msvc、独立target。

## 已通过的代码与接入检查

- 最终 `just ci`：80.4352 s，8个原PowerShell recipe命令退出码均0；命令仍来自正式 justfile，record-just-shell 只调用原PowerShell参数并保存退出码。前端73测试文件/300通过；Rust库546通过/6忽略，进程集成3通过；根Node203通过/0失败。逐门收据：`candidate-retained-20260930/just-ci-steps/`。
- release fmt、workspace/all-targets clippy `-D warnings`；raw-fold7通过/1忽略；service53通过/3忽略（含取消及整报告deadline）；bench12通过/1忽略。
- 双边A8/真实3tick接入smoke：同fixture、守恒、初/末原writer WAL/FULL、3个private/ingest/reader样本均通过，见 `retained-smoke-20260930/`。只证明接入，不代替正式矩阵或非空门。
- 正式runner防假PASS测试5项通过：非bool有限非负数、除数>0、缺字段/错误类型、零样本、source/dir/platform/PRAGMA及primary样本数，见 `minute-set-20260930/formal-runner-validation.*`。

## 最终非空生产证据

`retained-nonempty-20260930/` 与最终test身份绑定。每次实际运行1个精确ignored probe，native退出0、JSON存在、生产status=ok、tier=Raw。该fixture在之前的probe/oracle/hash中已被读取，没有物理冷页证明。

| 窗口 | 生产wall ms | connections | upload | download | series行 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 一分钟 | 4.6001 | 250 | 3497 | 11725 | 1 |
| 30天all | 6735.2968 | 2160000 | 151200035 | 507599545 | 720 |

probe的now=end；CLI容量使用真实Utc::now。单次最终候选非空结果不能证明完整F12同窗口双边非空比值门，也不替代21次容量。

## 正式证据状态

正式matrix在 `formal-matrix-20260930/` 执行，22次固定场景；primary后续为6次固定300秒测量。以各自summary与原native收据为准，当前代码检查通过不代表正式门通过。

A1000保持NOT_RUN，阶段前置证据未满足。AC3不完成。AC7全部应用文件与spool写量仍null，不能代入0或使用SQLite子集关闭全部文件门。AC8的30分钟峰值、WebView/后台worker和24h安装态保持未验证。下一阶段前置方案见 `next-stage-plan.md`：范围内工作继续使用既有用户授权，先完成强模型阶段审查和证据前置；新增范围另行批准。
