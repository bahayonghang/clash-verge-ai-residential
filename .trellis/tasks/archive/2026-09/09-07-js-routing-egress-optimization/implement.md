# 执行计划

状态：2026-09-07 用户后续明确批准按照本任务规划实施。

## 1. 实施入口

- [x] 确认用户批准最新 PRD/design，检查 git status、基线差异与 task 状态。
- [x] 阅读 frontend spec 与本任务 research；注意部分旧 spec 的原地修改/ci描述已被当前代码超越，详见研究报告。
- [x] 经批准后才激活任务；用 Trellis implement/check 子代理，主线程负责协调与最终核验。运行时、renderer、build-inputs 同属一条常量数据流，串行由同一实现代理负责，避免文件交叠。

## 2. 固定出口与核心控制（R1–R3）

- [x] 写实现前，用 task.json 的 baseline_commit 和虚构 Profile 固定默认配置投影 fixture（tests/fixtures/routing-default-v5.11.json）；按 validation.md 的规范化算法做独立差异验证，禁止从修改后的实现反向生成期望值。
- [x] 在 tests/regression.test.js 增加 F1 的反例、输入不变与合法 icon/hidden/幂等测试。
- [x] 在现有保留名检查中落实允许字段，buildAiGroup 构造固定对象。不得扩展到上游用户组。
- [x] 按 design.md 拆分三类核心数组，增加三个默认 true 常量。
- [x] 让 active 域名、process/IP 条件按设计生效；全量托管清理独立于开关。
- [x] 完成 V1–V10、V13；验证独立辅助开关以及用户规则例外，不只验证默认开关。

## 3. 跨文件接线（R4）

- [x] 更新 scripts/sync-local-config.js、公开 TOML example 和 Node constants 导出。
- [x] 用虚构临时 TOML 完成新键 true/false、类型/重复键错误、缺省值补全和不覆盖已有值测试。
- [x] 更新 skills/residential-rule-tuning/scripts/build-inputs.js 三个域名映射及 tests/install-agent-skills.test.js；验证集合归属及 24/12/12，不仅修改数字使测试通过。
- [x] 保留 unsupported 项和既有 JSON schema；不改 Rust CLI/数据库。不手改生成 local.js，不运行真实 render-local。

## 4. 文档与规范（R5）

- [x] 更新 docs/configuration.md、docs/en/configuration.md、docs/local-configuration.md、docs/en/local-configuration.md 四份开关表。现有测试逐项检查 SWITCH_CONFIG_FIELDS 与中文两张表，不能只改 local-configuration。
- [x] 同步 docs/routing-scope.md、docs/en/routing-scope.md 的核心类别、Google关闭组合、专属兜底与原 Profile 限定。
- [x] 同步 docs/dns-and-leak-model.md 及英文版本，写明 regex/DNS 和运行时固定出口边界。
- [x] README.md 仅在已有范围/开关描述需要同步时修改；skills/residential-rule-tuning/SKILL.md 与 reference.md 同步计数和归属，安装副本留给正常安装流程。
- [x] 更新本任务涉及的 frontend/index.md、component-guidelines.md、state-management.md/quality-guidelines.md 条目，纠正克隆输入及实际 ci 门禁；避免无关规范重写。
- [x] 在 CHANGELOG.md 的 Unreleased 记录三个新增开关、组所有权和非默认兜底变更。按 design.md 第8节延后统一版本号与正式发布；不更改 root package.json / SCRIPT_VERSION / README 当前版本，不把新行为描述成已发布功能。

## 5. 验证命令与验收

按变更风险逐步执行（Windows 支持的外部命令遵循 RTK）：

```powershell
rtk proxy node --test tests/regression.test.js tests/sync-local-config.test.js tests/install-agent-skills.test.js
rtk proxy npm run ci
rtk proxy just ci
rtk proxy git diff --check
rtk proxy python .trellis/scripts/task.py validate .trellis/tasks/09-07-js-routing-egress-optimization
```

`just ci` 当前依赖 monitor-check，涉及版本检查、监控前端依赖/检查及 Rust fmt/clippy/tests。若出现无关既有失败，记录命令/错误和影响，不私自扩大修复范围，也不能宣称全门通过。

文档构建使用 `npm --prefix docs run build`（Node 22+）。依赖已具备则执行；缺失环境记录阻塞，未经授权不增加新依赖。所有本地临时渲染产物使用虚构凭据，不能拷贝用户配置。

独立 check 子代理审查整个 diff、V1–V13、未知规则保留、唯一出口形状、清理完整性及默认等价性。根据最终范围做必要 secret 检查，完成后不重复扩大测试。

## 6. 宿主与交付边界

- [x] 记录自动化 PASSED/FAILED、未跑项与现场 UNVERIFIED，规划时不勾选现场结果。
- [x] 将隔离宿主配置加载、命中链、IP、UDP/DNS和业务验收作为独立证据；实际 Profile 切换、家宽故障注入需针对目标获得授权。
- [ ] 完成验证后按 Trellis 提交步骤确认提交清单，再做本地提交与收尾；不推送、不发布、不改变实际网络。

## 风险文件和回退点

最高风险为 clash-verge-ai-residential.js（顺序/所有权/清理）；其次 scripts/sync-local-config.js（常量映射）与 build-inputs.js（统计归属）。所有相关变更作为一个可回退的业务单元；先完成 Node 门禁再考虑现场。完整代码验证不构成节省流量或真实固定 IP 的证明。
