# 宿主兼容独立审查

日期：2026-11-06。审查角色：`trellis-check`。范围：已批准的 9 个改动文件，即 `tests/regression.test.js` 和中英文 configuration、local-configuration、multi-profile、troubleshooting 文档。未修改产品代码或文档。

## Findings (fixed)

无。已逐项核对任务 PRD、design、implement、check.jsonl 上下文、固定版本宿主合同研究及上述文件差异，未发现需要在授权范围内修复的问题。

## Findings (not fixed)

无代码或文档缺陷遗留。真实客户端加载、生产连接与平台验收的状态由父会话维护的 `validation.md` 记录，本审查不把它们提升为已通过。

## 审查结论

- 根脚本、路由域名、默认开关、DNS 算法与用户规则排序逻辑均未修改，AI-only 范围保持不变。
- 无 CommonJS 测试确实在独立 VM 中执行入口，检查 Node API 缺失、两种入口参数、输出等价、输入不变、幂等及非法 rule-providers 错误合同；额外测试覆盖缺少 Profile 名时跳过专属映射并走通用上游。测试注释明确不等同于 Boa、QuickJS 或 GUI 覆写验收。
- 文档区分 Verge 的带名调用和 FlClash 的单参数调用，限定固定版本，说明 FlClash 普通 Profile 的 Script 绑定和 Custom 模式边界。未把脚本输出、预览文件或 GUI 版本误当成已加载配置、真实流量或核心版本。
- 对应用后置 DNS 整表替换、进程模式、TUN、系统 DNS 和规则顺序的解释与宿主合同一致；保留规则没有被描述为保留其优先级。Windows/Ubuntu 与后续 macOS/Android 范围清晰。
- 中英文新增内容在行为、限制和验证要求上对应；没有引导将 FlClashCore 当成 Mihomo CLI 执行 `-v`、`-h`、`-t`。

## Verification

- Lint：PASS。根项目没有独立 lint；本审查执行 `git diff --check` 和 `node --check tests/regression.test.js`，均退出 0。
- TypeCheck：本次根 JavaScript/Markdown 改动不适用独立类型检查。父会话已报告完整 `just ci` 通过，包含监控端检查；未另行重复运行。
- Tests：PASS（父会话已完成）。完整门禁 267 个根测试、300 个前端测试、553 个 Rust 测试通过，6 个忽略；Windows/Ubuntu 各 104 个回归测试通过。本审查没有修复或新增变更，按调度要求未重复测试。
- 隔离核心证据：父会话报告 Windows/Ubuntu 各 4 份配置、36 次 DNS 查询通过；这不替代两客户端的真实宿主与生产服务验收。
- 文档构建由父会话负责；此记录不预先声明通过。
