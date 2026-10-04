# T03 已批准执行清单

用户于 2026-09-30 批准项目内实施；子任务按执行顺序启动。

## Order

1. 先固定授权、阶段与 V1/V2 文字；保留用户既有安全边界。
2. 修正 archive 引用、frontend 入口与版本日期，更新第一方来源地址。
3. 实现最小结构 checker，加入缺共享内容、反向导入、门名称变化、坏引用及已有覆盖中 checker 定义的结构约束损坏等负例。四个具名本机覆盖可选、不要求 Git 跟踪；缺失覆盖由 checker 跳过，缺失报告沿用既有环境诊断的 project.overrides 字段，不新增 checker 输出门。已有覆盖按定义的结构约束校验，非 ENOENT 读取错误失败；需要使用覆盖时继续核对任务路径一致性与授权限制。
4. 把 checker 接入 root check/test；审核源码差异并将最终业务源交给 T02 同步。

## Required Checks

- `node --check scripts/check-agent-contract.js`
- `node --test tests/check-agent-contract.test.js`
- `node scripts/check-agent-contract.js`
- `python .trellis/scripts/get_context.py --mode phase`
- `python .trellis/scripts/get_context.py --mode packages`
- `just ci`
- `just docs-build`
- `git diff --check`

新脚本、recipe 与测试命令是获批后的交付，不表示当前已存在或已通过。命令的预期失败、环境阻断和正式验收必须分别记录。仅最后一条 native 命令成功不能覆盖前面的失败。

## Review

强模型逐项核对 PRD、diff、检查证据和持久回写。共享文件按照父任务顺序串行处理；先读取前序改动，不得覆盖其他任务工作。

## Closure

全部 AC 有证据后才能声明本任务完成；缺少正式或动态证据保持未完成。提交、归档另需用户授权。

## 2026-10-01 合同修订

用户已批准保留本机覆盖不跟踪策略，更新任务合同和规范。四个本机路径及证据边界见 design.md。本轮合同同步仅修改本任务 prd.md、design.md、implement.md、task.json 与新增 research/local-override-policy-20261001.md；规范由主会话同步，不修改本机覆盖、Git 索引或产品。旧 review 和检查收据保持原样。

本次仅轻量编辑并静态核对。修订后的 AC5 定向检查、正式 just ci 与 docs-build 等待 T05 正式负载结束后执行；没有把 2026-09-30 的检查转为修订后通过。缺失报告和静态通过不作为原生角色加载证明。
