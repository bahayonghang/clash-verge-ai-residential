# T04 在初始化前识别实际 executable、版本和项目覆盖文件，避免旧 Trellis 或损坏包装器造成错误判断。

## Goal

在初始化前识别实际 executable、版本和项目覆盖文件，避免旧 Trellis 或损坏包装器造成错误判断。

## Evidence

F5/F8；Codex 首入口失败但应用内 CLI 成功；Trellis 0.6.17 与项目 0.7.0-beta.3 不匹配。 详细证据见 ../09-30-evergreen-five-harness-audit/research/audit.md。

## Requirements

- R1 解决本任务证据对应的问题，不扩大到相邻清理。
- R2 按下面可观察结果验收，保留失败与未验证边界。
- R3 实施后将批准结果回写并注明适用工具。

## Acceptance Criteria

- [x] AC1：只读诊断报告所选入口、版本、退出码和替代入口；区分 PATH 包装器故障、可用桌面二进制、项目配置和权限。
- [x] AC2：Trellis CLI 与 .trellis/.version 不同则拒绝执行 bootstrap；不调用全局 upgrade，不修改 PATH。
- [x] AC3：使用已批准隔离方案中明确选择且版本匹配的 Trellis 入口，在新候选目录执行既有 init 命令。四个具名本机覆盖可选、按需复制，不要求 Git 跟踪；逐项明确报告未选用或缺失状态。对已选用且获准复制的覆盖，SHA256 前后完全一致；存在或选用时仍符合任务路径一致性与授权限制。缺少可选覆盖本身不阻断 bootstrap，也不证明对应原生角色已加载。仅在本任务独立临时工具目录及 cache 获取固定版本；版本检查与 init 使用同一入口，不回落全局旧版本。
- [x] AC4：五平台必要资产存在；当前仓库 .template-hashes.json、全局 config、项目信任和私有文件未变化。
- [x] AC5：没有可用的匹配 CLI 时明确 BLOCKED，不以 --help 或文档宣称 bootstrap 通过。

2026-09-30 的 AC1–AC5 本地通过属于当日合同，四覆盖 tracked 与隔离目录中四覆盖 SHA256 一致的历史验收保留于 research/review.md 和原 bootstrap 收据。本机四个覆盖于 2026-10-01 仍存在。用户同日批准保留不跟踪策略并更新合同；修订后的 AC3 待定向检查和正式门，旧 bootstrap 不覆盖缺失可选覆盖的新场景。

2026-10-02 缺失四可选覆盖的隔离复验：AC3 为 **PARTIAL**，保持未勾选。固定 CLI/core `0.7.0-beta.3` 使用同一入口完成版本门，init 退出 0；init 前 checker 退出 0，默认覆盖生成后 checker 首次退出 1，共 5 项 Kimi AUTH/DISPATCH 与 2 项 Codex DEPTH 说明缺失。四项 init 前均 missing、未选用、未复制，init 后均生成默认内容；没有修补或复制本机覆盖掩盖失败。候选采用 67 个明确公开文件，证据清单含 52 个文件；67 个复制文件与公开来源、9 个保护文件及 index/HEAD/branch 前后相同。结果为 **INIT_PASS / CONTRACT_FAIL**，不代表完整 fresh checkout 或原生角色加载。证据见 research/bootstrap-missing-overrides-20261002/result.md 与 evidence-manifest.json；旧收据保持历史范围，任务仍为 in_progress。

## Dependencies

T03 的合同和版本说明稳定后实施。T02 完成后记录业务 skill 校验；bootstrap 本身不能替代业务 skill 安装。

## Authorization

用户于 2026-09-30 明确批准项目内实施。具体文件及步骤见 design.md / implement.md。本任务采用原设计的隔离固定版本执行器，工具依赖获取仅限本任务新临时目录及独立 cache。全局配置、全局工具安装或升级、应用安装、真实凭据/数据库、push、PR、远端 workflow、提交与归档不在授权内。

用户于 2026-10-01 明确批准「保留当前不跟踪策略，更新任务合同和规范」。本次追加范围仅为本任务的本机覆盖策略合同同步及对应规范，由主会话同步规范；不扩大 T01/T02 或本机覆盖内容的修改授权。详见 research/local-override-policy-20261001.md。

## 2026-10-02 公开模板路线验收

独立审查及真实隔离初始化通过，补齐AC3。固定0.7.0-beta.3同JS入口version/init与候选checker均退出0；四公开覆盖字节保留，五平台资产存在，当前本机覆盖、原template/index/HEAD/branch保持。79项定向fixture、51项T06合成、完整justci和docs-build均通过。历史INIT_PASS/CONTRACT_FAIL保留原候选范围；本轮只证明基本初始化，不声明真实原生角色加载。证据见父任务research/operational-review-20261003T023405Z/。
