# T04 基本初始化恢复实施方案

日期：2026-10-02，America/Chicago。用户最新要求继续其它改造并以正常运行为目标，不等待竞争负载。主会话审查默认生成文件与原 checker 失败后，批准以下项目内最小修复范围。

## 文件范围

- 新增 scripts/bootstrap-harnesses.js。
- 新增 scripts/harness-templates/codex-config.toml 与 kimi-trellis-implement.md、kimi-trellis-check.md、kimi-trellis-research.md。
- 新增 tests/bootstrap-harnesses.test.js。
- package.json 仅接入新脚本语法检查与新 fixture 测试；justfile 仅添加显式 bootstrap 入口。
- docs/agents/harnesses.md 的初始化操作说明；规范由主会话回写。

现有 scripts/check-harness-environment.js 保持只读；check-agent-contract.js 不放宽。当前四份本机覆盖保持 ignored 且不修改内容。原 bootstrap 首失败、模板输出、收据及全部旧证据保留。

## 执行合同

显式指定匹配版本的 Trellis 可执行入口与新隔离目标目录，使用同一入口核对 version 和执行原 init 参数。不自动安装、升级、改变 PATH、trust、账户、模型或权限。拒绝当前工作树及不符合隔离条件的目标。先核对目标路径与链接边界，失败前不覆盖目标文件。

公开模板在 init 前仅补齐缺失的四个具名覆盖；已有文件逐字保留。Codex 模板保留 max_depth=1 并明确 V1/V2 与提示词限制；Kimi 模板实际限制自修授权、任务路径核对和内建 coder 派发，不保留默认无条件自修或错误角色说明。模板不含机器路径、凭据或全局模型选择。

初始化后分别记录 init 退出、原 checker 退出及覆盖前后 hash；任何原错误不写成成功。隔离运行成功只证明初始化及本地合同，不补全真实客户端角色、hook、权限或子代理验收。

## 验证

fixture 覆盖缺失文件部署、已有文件字节保留、版本不合与入口失败、当前根/越界或链接路径拒绝、init 失败传播、生成资产/合同复查及幂等行为。使用 node:test 与可注入执行器，不依赖全局配置或网络。实施与独立检查后运行实际 just ci、just docs-build 和秘密扫描；已有匹配入口可用时再做一次新隔离 native init，保留独立原始收据。

本范围属于用户本轮继续实施正常运行路径的项目内授权。没有提交、归档或外部交付授权。
