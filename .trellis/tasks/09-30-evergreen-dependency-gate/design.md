# T01 设计

## Mechanism

优先在既有 semver 范围内更新 brace-expansion 和 js-yaml 的 lock 解析。执行时重新核对公告与可用版本。新增独立 dependency-audit recipe；monitor/docs 的 CI 安装后各运行明确的 high 级 npm 审计。Rust cargo-audit 保留独立检查和 warning 分类，不将 glib 的其它 target 警告描述为 Windows 运行漏洞。无需给根脚本添加第三方依赖。

## Owned Files

- residential-monitor/package-lock.json
- residential-monitor/package.json（仅现有范围无法修复时）
- justfile
- .github/workflows/ci.yml
- tests/sync-monitor-version.test.js
- AGENTS.md
- README.md
- .trellis/spec/residential-monitor/frontend/index.md

## Model And Harness

强模型负责依赖链、安全边界与最终审查；较低成本模型可按批准的版本范围修改 lock、测试和文档。 执行工具按父任务五工具矩阵选择；工具原生能力不扩展授权。

## Writeback

AGENTS.md/README 的验证边界和 monitor frontend spec，适用五工具。

## Rollback

回退本项锁文件、CI 和 recipe；保留初次 audit 失败记录。不使用 npm audit fix --force。
