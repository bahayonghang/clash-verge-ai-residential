# 改造设计与批准边界

状态：已批准项目内实施（2026-09-30）。依据：research/audit.md。子任务按执行顺序启动。

## 责任划分

- AGENTS.md：五工具共享的产品、授权与质量合同。CLAUDE.md：Claude loader。
- .trellis/workflow.md：阶段路由与批准边界；不替代 harness 原生权限。
- docs/agents/harnesses.md：版本、平台差异、bootstrap、fallback、运行证据。
- skills/residential-rule-tuning/：业务 skill 单一源。ignored 平台目录只做受控交付。
- justfile / package.json / CI：可执行质量门。任务 research/：版本化的审查与验收收据。

## 任务映射

| 子任务 | 优先级 | 问题 | 主要文件 | 成功条件 |
| --- | --- | --- | --- | --- |
| T01 evergreen-dependency-gate | P1 | F1 | residential-monitor/package-lock.json；必要时 package.json；justfile；ci.yml；既有 CI 合同测试 | 两个 high 清除，新增安全门失败可传播，just ci/docs/audit 通过 |
| T02 evergreen-skill-sync | P1 | F2 | skill 源说明；安装说明；tests/install-agent-skills.test.js；七个已列项目副本 | 21 项已审查同步，有备份，实际七目标一致，26/13/13，二次安装无写入 |
| T03 evergreen-harness-contract | P1 | F3/F4/F8 | AGENTS.md；CLAUDE.md（按需）；workflow；frontend/index；harnesses；Codex 配置注释；相关 skill；说明检查脚本/测试 | 共享授权明确，V1/V2 保证准确，负向说明测试阻断漂移 |
| T04 evergreen-bootstrap-check | P2 | F5/F8 | harnesses；新 scripts/check-harness-environment.js 及测试；justfile | 报告实际 executable/version，版本不合则停止；本机覆盖可选，隔离 bootstrap 保留已复制覆盖的字节 |
| T05 evergreen-performance-evidence | P1 | F6 | c3/raw_fold.rs、sql.rs、service.rs；bench/facade.rs；c3/bench 测试；storage/sqlite-contract | 先重建同窗口证据；最终保留原 10 秒、110%、守恒和 21 次容量门 |
| T06 evergreen-runtime-validation | P2 | F7 | harnesses 运行矩阵；本任务/子任务 research；必要的已有 spec 验收说明 | 五工具分别有新会话证据；当前提交 hosted 证据单独补齐，不挪用历史成功 |

## 决策

1. 保持本地 TOML 的现有禁止手改默认。skill 可以形成 routing.* 建议，由用户应用；自动编辑需要用户另行明确覆盖授权。不得借本轮文档改造扩大凭据或私有配置权限。
2. 只读审查仅写获准 research/ 与任务产物。trellis-check 的自动修复仅在已批准实施阶段、已列文件范围内生效。
3. 不改用户全局模型、PATH、hook trust 或 project trust。Codex 可用二进制路径作为当前环境证据，不把含安装 hash 的机器路径写成通用命令。
4. 本机 Codex 配置存在时检查 max_depth=1 对 V1 的防护，补充 V2 不受该字段控制的说明。2026-10-01 用户批准四份 Codex/Kimi 覆盖继续不进入 Git；新 checkout 可缺失，使用者按需复制。检查器校验已存在覆盖，诊断明确报告缺失。任何额外机制必须按实际后端验证；不能以提示词声称运行时硬限制。
5. 不把 ignored 安装目录要求塞入 clean CI。CI 验证真实 payload 的临时七目标，开发环境单列实际安装态检查。
6. T01 最小锁文件修补；不升级 ESLint/TypeScript major，不关闭安全告警。审计服务不可达应记为检查错误，不能记为无漏洞。
7. T05 先做测量设计与 baseline 恢复。长窗口算法替换只有证据足够且设计仍在批准范围内才实施；改变 schema、核算合同或文件范围需追加批准。

## 顺序与共享文件

建议 T01 → T03 → T02 → T04 → T05 → T06。T05 的测量研究可在独立资源条件下提前；正式性能对照期间不并行运行其它构建/压测。T02 在 T03 完成业务 skill 源合同改动后做最终同步；T04 依赖 T03 的合同；T06 依赖所有涉及的候选改动。

package.json、justfile、ci.yml、harnesses.md 和业务 SKILL.md 按上述顺序串行编辑。每任务启动前读取前序结果，禁止覆盖他人改动。批准后的实施与独立审查按适用流程委派，明确任务首行、独占文件和模型职责。

## 回写与适用工具

| 项 | 获批后持久位置 | 适用范围 |
| --- | --- | --- |
| 安全门与依赖结果 | AGENTS.md 验证段、README 本地验证、monitor frontend spec、CI | 五工具共享 |
| skill 生命周期 | skills/residential-rule-tuning/SKILL.md、reference.md、docs/agents/residential-rule-tuning.md | 五工具及 .agents/.cursor 投递目录 |
| 授权与审查边界 | AGENTS.md、workflow、harnesses.md；本机 Kimi 角色的条件校验说明 | 五工具，逐平台标注；本机覆盖保持 ignored |
| Codex V1/V2 | harnesses.md、本机 .codex/config.toml 的条件校验说明 | Codex；不写用户配置或自动创建覆盖 |
| bootstrap 和启动诊断 | harnesses.md、环境检查命令 | 五工具；Codex/Trellis 故障分开 |
| 性能合同与结果 | storage/sqlite-contract.md、相关 backend spec、任务 evidence | 五工具共用的 ResiWatch 工作 |
| runtime 证据 | harnesses.md 的版本/结果表、T06 research | 每工具独立，不跨工具继承 PASS |

不向个人原生记忆或 Basic Memory 自动写入。若用户明确选择团队知识库，再先查重后写入。

## 回退

每项按独占文件回退，保留初次失败日志。skill 使用现有 .bak-UTC 备份。T04 的 bootstrap 只在新临时目录中运行，不重写当前 .template-hashes.json。性能实验使用生成/只读隔离库和隔离二进制，不覆盖安装态。
