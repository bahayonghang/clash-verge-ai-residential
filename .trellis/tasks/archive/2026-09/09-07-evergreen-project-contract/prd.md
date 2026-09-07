# 统一项目说明与规范导航

## Goal
让五套工具读到同一份准确项目合同，避免把 ResiWatch 当成无依赖单脚本。

## Evidence
- `CLAUDE.md:6-9,12,35-36`：单脚本范围扩大到仓库、误称 just ci=npm run ci、未区分 React/Rust 工具链。
- `AGENTS.md:23`：仅用 @CLAUDE.md 转接项目说明，非五工具共同加载协议。
- `.trellis/scripts/common/packages_context.py:28-40` 首层枚举 residential-monitor，但对应 index 不存在；规范在 backend/frontend/storage。

## Requirements
- R1 AGENTS 为项目内共享事实源，CLAUDE 为兼容入口，不复制用户全局规则。
- R2 明确三套工具链、路径范围、检查、凭据/生成物和审批边界。
- R3 增加最小监控端规范导航，不迁移目录或改 Trellis 引擎。
- R4 按适用工具说明加载，不把文字对齐当实际新会话验证。

## Acceptance Criteria
- [x] AC1 / R1：AGENTS 自包含重要事实，不依赖隐式 @ 展开；CLAUDE 单向导入，无循环。
- [x] AC2 / R2：root Node18+、docs Node22+、monitor React/Tauri/Rust 命令及 just ci 范围准确。
- [x] AC3 / R3：get_context 输出的两个首层 spec 均有可读 index；监控索引指向三个真实层。
- [x] AC4 / R1–R4：强模型逐条核对授权/命令/loader，docs build、safety scan、链接检查通过。
- [x] AC5 / R4：只回写项目说明，C3 单独记录各客户端动态证据。

## Out of scope
用户全局设置、Basic Memory 迁移、新平台体系、通用规则验证器。

## Dependencies
最终命令表在 C1 后收敛；C3 依赖本项共享合同。
