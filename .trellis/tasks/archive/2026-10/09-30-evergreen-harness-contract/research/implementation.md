# T03 实施与验收记录

日期：2026-09-30。已派发 `trellis-implement` 直接实施，没有递归委派。用户已批准项目内范围。T01 的已验收命令和其它任务证据保持原状。T03 的合同与代码已于 12:57 UTC 验证后释放给主会话；提交、归档与外部操作未执行。

## 交付

| 文件 | 改动与适用工具 |
| --- | --- |
| `AGENTS.md` | 五工具共享授权：只读审查不修复产品/配置；self-fix 须处于已批准任务和文件范围；个人 TOML 默认形成建议，由用户应用；例外必须来自用户明确授权。保留 T01 的完整依赖审计门。 |
| `.trellis/workflow.md` | 五工具的共享 review 条件、两种执行 breadcrumb 和 Phase 2.2 同步授权条件。补充注入路径与派发目标不一致时的主会话确认和显式 pull。修正本地不存在的上游 spec/parser 引用。 |
| `.trellis/spec/frontend/index.md` | 共享入口改为 AGENTS；本地 TOML 授权引用共享合同。 |
| `.trellis/spec/frontend/quality-guidelines.md` | 同步七条 Windows native step、docs 完整 npm audit 和独立 dependency-audit；记录结构检查器与语义/运行证据边界。 |
| `docs/agents/harnesses.md` | 更新 2026-09-30 安装版本快照、正确归档路径、第一方依据、授权与 V1/V2 边界；保留 fresh-session 未验证状态，说明 bootstrap 版本不匹配须停止。 |
| `skills/residential-rule-tuning/SKILL.md` | 五工具业务源统一为建议由用户应用；不由 skill 创建 local TOML 例外。交给 T02 同步七个项目目标。 |
| `.codex/config.toml` | 只改注释。说明 max_depth 的 V1 范围、V2 忽略和提示词保证边界；保留 max_depth=1，无 feature/权限/信任变更。 |
| `.kimi-code/skills/trellis-check/SKILL.md` | Kimi 角色明确获批 self-fix 和只读边界；独立 reviewer 进一步修正 Step 1，使派发/注入/current 路径核对不会被 stop-at-first 跳过。 |
| `scripts/check-agent-contract.js` | 新增 Node 标准库 checker：共享入口、单向导入、授权标记、质量门、四 override、阶段标签和具体任务/脚本路径。失败非零退出。 |
| `tests/check-agent-contract.test.js` | 最终 39 项测试。覆盖正常仓库、LF/CRLF、缺入口/反向导入、批准与范围丢失、Kimi 冲突流程、门名/参数漂移、缺覆盖、坏路径、CLI 0/1 退出。 |
| `package.json` | `check:agents`、`check` 的语法与结构检查、`test` 的回归套件接线。 |

`CLAUDE.md` 已正确单向导入，保持不变。Kimi implement/research 两份 override、`.trellis/.template-hashes.json` 和 Rust 产品源码未修改。没有编辑其它被忽略的生成角色或业务 skill 安装副本。没有读取或修改真实凭据、本地 TOML/JS、生产数据库或控制器。

## 验证结果

每组 `.result.json` 记录单条命令、UTC 时间与命令退出码。`.log.gz` 保留原始输出字节；`.log` 仅移除 ANSI 与行尾空白并转为 LF。

| 检查 | 结果 | 收据 |
| --- | --- | --- |
| 首次 focused test | PASS，37/37，直接工具结果 | 后续正式门覆盖所有最终用例 |
| 独立 reviewer 修正后 focused test | PASS，39/39，exit 0 | `review-focused.*` |
| 首次 `just ci` | FAIL，exit 1；Rust 单元 541 pass / 1 fail / 6 ignored | `product.*`；失败保持 |
| 一次同版本 Rust 定向复测 | PASS，1 test；首轮 30 行 / 105.9169ms；reopen 清空 | `rust-targeted.*` |
| 单独 `npm run ci`（reviewer 的两个新测试之前） | npm exit 0，180/180；collector 打印发生编码错误，见调查说明 | `root-ci.*` |
| 经主会话批准的单次完整复跑 `just ci` | PASS，exit 0；Vitest 73 文件/300 tests；Rust 单元 542 pass/6 ignored；kill_gate 集成 3 pass；Rust doc tests 0；Node 182/182；模板安全扫描通过 | `product-rerun.*` |
| `node --check scripts/check-agent-contract.js`、测试文件语法和实际结构 checker | PASS；由最终 root check 执行，含在完整门中 | `product-rerun.log:682–687` |
| `just docs-build` | PASS，exit 0；使用已有 docs 依赖，不声明本任务全新安装 | `docs-build.*` |
| `get_context.py --mode phase` / `--mode packages` | PASS，exit 0 | `phase.*` / `packages.*` |
| Phase 2.2 的 Codex / Kimi Code 实际段落抽取 | PASS，exit 0；正确参数为 codex / kimi-code，含授权、角色与 fallback | `phase-codex.*` / `phase-kimi-code.*` |
| 结构与范围核对 | Codex TOML 解析语义与 HEAD 一致；workflow 标签序列与 HEAD 一致；保留文件未改；最终文件 hashes 留档 | `scope-integrity.json` / reviewer 的 `review-structure.json` |
| 最终空白和静态收尾 | PASS，exit 0；diff、task validate、33 个新增文本空白检查和正式门文件 hash 核对通过 | `final-static.*` |

首次 Rust 失败的直接路径与残余不确定性、collector 编码问题和 Kimi 平台参数记录在 `failure-investigation.md`。正式复跑通过不等于该时间敏感失败已修复。T03 没有修改清理逻辑或 250ms 预算。

## AC 映射

- AC1：AGENTS 保持自包含共享权威，CLAUDE 单向导入保持；frontend 入口和正反测试完成。
- AC2：共享合同与业务源均要求建议由用户应用，例外须用户明确授权；生成 JS 禁止手改。对应负例通过。
- AC3：AGENTS、workflow、Kimi check、harness docs 与业务源统一只读/self-fix 边界，保留信任与权限规则。路径冲突流程有回归。没有借角色权限扩大实施范围。
- AC4：Codex 配置语义保持，max_depth=1 不变。说明引用本轮保存的当前及 rust-v0.159.2 schema 证据，并明确 V2 不受该字段控制。未切换后端、用户 feature 或 hook trust。
- AC5：正常仓库、LF/CRLF、所有失败 fixture、CLI 退出码和 root 接线通过；四 override tracked 状态由独立 reviewer 核对。动态客户端行为不由 checker 证明。

实施侧已完成上述 AC 的静态与本地检查。独立审查结论由 `review.md` 记录，父任务整体验收与 T02/T04/T05/T06 各自继续。

## 剩余边界

- 五客户端 fresh-session、实际 hook/extension/child 和当前修改的 hosted CI 仍归 T06；本任务未声称这些检查通过。
- T02 尚需交付业务 skill 的实际安装态，T04 尚需匹配版本隔离 bootstrap。
- Rust 的首次墙钟预算中断原因未查明。源码未修复；首次 failure、定向 PASS 与正式复跑 PASS 同时保留。
- Rust 6 项 ignored 不计入通过。真实桌面、凭据写入、容量/性能和其它硬件/安装验收均保持原边界。
- `task.py validate` 返回 0，并提示 `.trellis/workflow.md` 为 44244 bytes，超过 32768 bytes 的单文件注入上限。完整文本显式读取、phase 抽取和 fallback 仍需按合同执行。没有修改全局/项目注入限额，也没有把 marker 当作完整注入证明。
