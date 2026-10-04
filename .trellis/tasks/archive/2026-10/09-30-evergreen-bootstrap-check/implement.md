# T04 已批准执行清单

## 2026-10-02 继续实施

按 research/operational-bootstrap-plan-20261002.md 实施独立 bootstrap 与公开模板。执行顺序为 fixture 与语法检查、独立审查、完整 just ci/docs-build、新隔离 native init。已有竞争负载仅记录，不再作为本轮基本初始化与本地检查的启动阻断。原首失败保留，当前四本机覆盖不修改。不得以本轮基本运行结果补全正式性能或客户端动态证据。

用户于 2026-09-30 批准项目内实施；子任务按执行顺序启动。

## Order

1. 先验证路径优先级、包安装态和匹配版本的可用性。
2. 实现 fixture 驱动的只读诊断；测试 missing/broken/wrong-version/available 分别呈现。
3. 在新临时目录放入 Git 跟踪的共享合同。四个具名本机覆盖可选、不要求 Git 跟踪，按需要和既有授权复制所选覆盖；逐项明确报告未选用或缺失。对已选用且复制的文件计算 SHA256，并核对任务路径一致性与授权限制；不得在当前工作树 init。
4. 用匹配版本运行 trellis init --claude --codex --grok --kimi --omp --skip-existing -y；回读 hash、生成资产与忽略范围。
5. 回写可重复命令、前提和各工具发现状态。

## Required Checks

- `node --check scripts/check-harness-environment.js`
- `node --test tests/check-harness-environment.test.js`
- `node scripts/check-harness-environment.js（本项新增的只读诊断）`
- `匹配版本 Trellis 的隔离 init；已选用且复制的覆盖 SHA256 前后比较，未选用或缺失逐项报告`
- `git check-ignore -v 检查平台文件仍 ignored；不要求四个本机覆盖 tracked`
- `just ci`
- `just docs-build`
- `git diff --check`

新脚本、recipe 与测试命令是获批后的交付，不表示当前已存在或已通过。命令的预期失败、环境阻断和正式验收必须分别记录。仅最后一条 native 命令成功不能覆盖前面的失败。

## Review

强模型逐项核对 PRD、diff、检查证据和持久回写。共享文件按照父任务顺序串行处理；先读取前序改动，不得覆盖其他任务工作。

## Closure

全部 AC 有证据后才能声明本任务完成；缺少正式或动态证据保持未完成。提交、归档另需用户授权。

## 2026-10-01 合同修订

用户已批准保留本机覆盖不跟踪策略，更新任务合同和规范。四个本机路径及证据边界见 design.md。本轮合同同步仅修改本任务 prd.md、design.md、implement.md、task.json 与新增 research/local-override-policy-20261001.md；规范由主会话同步，不修改本机覆盖、Git 索引或产品。旧 review、bootstrap-prerequisites 和 bootstrap 收据保持原样。

本次仅轻量编辑并静态核对。修订后的 AC3 定向检查与既定正式门等待 T05 正式负载结束后执行；缺少可选覆盖的记录不等于 bootstrap 失败，也不能补充原生角色加载证明。2026-09-30 的获取、init 和测试结果仍只对应当日候选及合同，不重复获取工具或重跑 init 来改写旧结果。
