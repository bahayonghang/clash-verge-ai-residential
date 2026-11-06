# 宿主兼容执行清单

前提：用户批准最新规划后才运行 task.py start。所有修改遵循 PRD File Scope。

1. [ ] 记录 Windows/Ubuntu 两客户端及实际核心版本；无环境先写清缺口。
2. [x] 固定来源，核对 Verge 字段恢复和 DNS 订阅设置选择，FlClash script Rust 入口及应用覆写的完整顺序。见 `research/host-contracts.md`。
3. [x] 盘点已有 regression coverage，列出确实缺少的测试；关注可选 profileName、无 module、null 输入与错误结构、幂等/不变、最后生效 DNS、查找进程。新增 3 个入口边界测试，未伪造完整宿主模拟。
4. [x] 用最小虚构 Profile 核对入口合同；未复现需修改根脚本的缺陷，根脚本保持不变。
5. [x] 更新中英文客户端接入/设置/故障排查说明。明确当前脚本不会赋予用户例外最高优先级。
6. [x] 运行 node --test tests/regression.test.js：Windows 与父会话 Ubuntu 均 104/104 PASS；未改 renderer/schema。
7. [x] just ci 与 just docs-build 通过，详见 research/validation.md；未降低 gate。
8. [x] 两平台随附 Verge Mihomo 均通过隔离配置检查和 DNS/SOCKS 探针；FlClashCore 不提供通用 CLI，未冒用其参数或生产 IPC。详见 research/validation.md。
9. [x] Trellis check 已完成，见 research/check.md；版本与剩余实机限制已交给运行子任务。

产品或依赖修改扩大时回到父任务。没有四组合实机结果时只能报告代码合同和已测范围，不报告全面兼容。
