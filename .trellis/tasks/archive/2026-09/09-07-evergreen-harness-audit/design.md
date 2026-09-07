# 改造计划与审查结论

状态：**子任务已实施并归档**。审查基线：`dev@5578576787dba73ba1b96985a2fc408fc246bfed`，2026-09-07。五个子任务均为 `completed`，目录在 `.trellis/tasks/archive/2026-09/`。父任务 AC4 关闭对照：`implement.md` 与 `research/ac4-writeback.md`。静态/本地完成。hosted CI 为 UNVERIFIED。五客户端 live smoke 为 UNVERIFIED。不得宣称五工具运行时对齐。

## 规划基线结论

规划时（`dev@5578576787dba73ba1b96985a2fc408fc246bfed`）现有自动测试没有失败：Node125、前端284（69文件）、Rust435通过/1忽略，类型/lint/build/clippy与docs通过。当时发现的断点是CI失败传播、项目规则加载与skill交付。根 `npm run ci` 通过不能证明整套harness交付成功。下表 F1–F7 是规划编号，不是关闭后仍开放的缺陷。

| ID | 优先级 | 证据与影响 | 所属子任务 |
|---|---|---|---|
| F1 | P1 | ci.yml:75–83 的pwsh多命令step：exit7后再exit0整体成功；中途失败可能导致Required checks假绿 | C1 |
| F2 | P1 | AGENTS.md:23只用@CLAUDE.md转接；五工具没有共同的@加载合同，关键项目规则不能只放在Claude入口 | C2 |
| F3 | P2 | CLAUDE.md:6–12,35–36过时；just ci≠npm run ci；monitor首层spec/index缺失导致启动导航断点 | C2 |
| F4 | P2 | workflow.md:104–106及Kimi委派段与本地适配/当前能力不一致；修ignored文件不会随clone交付 | C3 |
| F5 | P2 | 真实skill --check exit1，共21文件漂移；副本缺3core且Grok映射错误，源测试仍绿 | C4 |
| F6 | P2 | live main protection linear-history=false，与quality-guidelines.md:103要求true冲突 | C5，单独远端授权 |
| F7 | P2 | hosted required gate不含docs；当前docs可构建，属于预防性质量门改进 | C1 |

P0：规划时未发现。Action full-SHA pinning为P3可选加固，延期，不新增依赖管理或远端policy任务。

F1–F7 已由对应归档子任务处理。本地静态结果见 [research/ac4-writeback.md](research/ac4-writeback.md)。hosted CI 与五客户端 live smoke 仍 UNVERIFIED。

## 任务与文件/检查地图

| 子任务 | 修改文件（具体展开见子design） | 必须通过的检查 | 依赖/分工 |
|---|---|---|---|
| **C1 / P1** [CI gates](../archive/2026-09/09-07-evergreen-ci-gates/prd.md) | .github/workflows/ci.yml；必要时现有sync-monitor-version测试 | 负向exit传播；root ci；monitor前端/Rust；docs；对应SHA hosted矩阵及aggregate | 已归档。本地 just ci / docs-build 退出 0。hosted UNVERIFIED |
| **C2 / P1** [Project contract](../archive/2026-09/09-07-evergreen-project-contract/prd.md) | AGENTS.md、CLAUDE.md、README验证段、monitor新index、frontend quality spec的CI段 | 命令/范围/引用人工核对；packages索引；docs build；safety；diff检查 | 已归档。五工具共享 AGENTS。动态加载 UNVERIFIED |
| **C3 / P2** [Harness adapters](../archive/2026-09/09-07-evergreen-harness-adapters/prd.md) | .trellis/workflow.md、docs/agents/harnesses.md（新）、.gitignore、4个窄适配override | 隔离bootstrap保留override；5条分平台加载/委派审查；已授权可用客户端smoke；其余UNVERIFIED | 已归档。静态完成。五客户端 live smoke UNVERIFIED |
| **C4 / P2** [Skill delivery](../archive/2026-09/09-07-evergreen-skill-delivery/prd.md) | install-agent-skills测试、业务skill源说明、安装文档、7目录21个本地副本 | 临时7目标真实payload/幂等/备份/冲突；root ci；本机--check=0；24/12/12映射 | 已归档。本机 --check 退出 0。客户端加载 UNVERIFIED |
| **C5 / P2** [Branch governance](../archive/2026-09/09-07-evergreen-branch-governance/prd.md) | main branch protection（外部）、quality spec治理段、前后快照 | 新鲜GET→获批精确变更→GET全字段对比；下次获授权PR证据 | 已归档。linear-history=true。下次 PR UNVERIFIED |

文件所有权：C1不改justfile/共享说明；C2只改quality spec的工具链/CI段；C5稍后负责治理段；C3不复制业务skill源；C4不改Trellis role skills。不能并行改同一文件。

## 五工具能力边界与成本分工

能力是harness的执行/加载机制，不等于模型智力。以下“适合”是基于本项目任务的调度建议，不是实测模型排行榜；完整第一方出处与具体本地文件在 [harness-audit.md](research/harness-audit.md)。

| 工具 | 本项目使用边界 | 更适合强模型规划/审查 | 可下放给较便宜模型的执行 |
|---|---|---|---|
| Claude Code | CLAUDE入口、项目hooks/agents；原生@导入仅在其适用范围使用 | 共享规则合同、复杂变更独立审查、CI方案反证 | 已指定文案、YAML和测试fixture修改 |
| Codex | AGENTS、TOML代理、当前工具权限；hook信任/表面差异不能由文件存在证明 | CI日志→shell根因、Rust/SQLite/IPC跨层审查、整体验收 | 确定性修补；主模型批准范围与AC后委派 |
| Grok Build | .grok/agents；本项目Trellis显式pull上下文 | 路由/网络边界与文档的独立复核（需真实资料） | 范围明确的JS/文档子任务；不假设Claude hook语法通用 |
| Kimi Code | 当前采用built-in coder+role skill pull；平台能力不止这一种 | 已聚焦子任务的设计/审查，按可用强模型配置 | 固定文件的文案/fixture/重复修正；不得自行换委派方式 |
| OMP | .omp agents与extension；provider/model可配置，OMP指Oh My Pi | 多provider下的任务拆分与独立审查安排 | 独占文件的并行机械修改；聚合验收仍归强模型 |

低价下放条件：根因已确认、文件范围独占、期望行为与失败用例明确、不涉及权限/凭据/真实路由/远端治理。出现新根因、额外文件或无法解释的测试失败时交回强模型，不继续凭猜测扩大修复。模型按账户实际可用列表和报价选择，本轮不写死价格、不把某harness品牌当成低价模型、不做全局模型降档。

## 已修复历史与证据边界

两次真实Windows CI失败分别是SKILL frontmatter的CRLF和TOML fixture删除行的CRLF，已有修复提交与后续成功run，不重开子任务。历史bootstrap的checksum与workflows权限错误属于已退役流程。见 [CI审查](research/ci-audit.md)。

当前本地比GitHub dev领先31提交；最新hosted绿色覆盖旧SHA，不证明本地这31提交。Credential Manager写入被ignore；真实Clash/Mihomo、WebView、NSIS、五工具新会话、30天库/24小时soak未冒充测试通过。详见 [本地基线](research/local-baseline.md)。

## 批准与回写

C1–C5 均已实施并归档。父任务 AC4 已对照实际文件、检查记录与规则。团队知识库不是本次选定写入目标。逐项落点、适用工具、证据与残留见 [research/ac4-writeback.md](research/ac4-writeback.md)。

静态/本地完成。hosted CI 为 UNVERIFIED。五客户端 live smoke 为 UNVERIFIED。不得宣称五工具运行时对齐。不归档父任务。不提交。
