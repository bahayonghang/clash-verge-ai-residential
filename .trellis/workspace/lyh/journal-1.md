# Journal - lyh (Part 1)

> AI development session journal
> Started: 2026-07-23

---



## Session 1: 完善 Clash Verge 前端开发规范

**Date**: 2026-07-23
**Task**: 完善 Clash Verge 前端开发规范
**Branch**: `main`

### Summary

基于真实 CommonJS 扩展、配置渲染器与回归测试补齐七份前端规范；完成独立检查与 just ci 验证，并归档 bootstrap 任务。

### Main Changes

- Detailed change bullets were not supplied; see the summary above.

### Git Commits

| Hash | Message |
|------|---------|
| `cbc99d8` | (see git log) |

### Testing

- Validation was not recorded for this session.

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 2: 完成 v5.5 路由与 TOML 配置

**Date**: 2026-07-23
**Task**: 完成 v5.5 路由与 TOML 配置
**Branch**: `main`

### Summary

收窄默认 AI 路由，增加本地 TOML 开关、渲染校验和完整使用文档

### Main Changes

- Detailed change bullets were not supplied; see the summary above.

### Git Commits

| Hash | Message |
|------|---------|
| `615b7f9` | (see git log) |
| `be0fb3f` | (see git log) |
| `fb30f69` | (see git log) |

### Testing

- Validation was not recorded for this session.

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 3: 完善测试门禁与 main 分支保护

**Date**: 2026-07-23
**Task**: 完善测试门禁与 main 分支保护
**Branch**: `chore/branch-protection-closeout`

### Summary

将测试迁移到 node:test，增加安全扫描回归与跨平台稳定门禁；通过 PR #2 合入并为 main 启用严格、app-bound 的 Required checks 分支保护；通过受保护 PR #3 验证 BLOCKED 到 CLEAN 的维护闭环。

### Main Changes

- Detailed change bullets were not supplied; see the summary above.

### Git Commits

| Hash | Message |
|------|---------|
| `9b2ed57896e1694ca987a2eb2e41008800ddab5a` | (see git log) |
| `60d87226796818989cca39eea706416602e3ab38` | (see git log) |
| `28e940a` | (see git log) |

### Testing

- Validation was not recorded for this session.

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 4: 补齐 ChatGPT 官方 exact 主机并归档审计任务

**Date**: 2026-08-17
**Task**: 补齐 ChatGPT 官方 exact 主机并归档审计任务
**Branch**: `dev`

### Summary

对照官方 9247338 与社区规则集后，仅以 exact 补齐五个 ChatGPT 主机；suffix 方案因过宽被否决。npm run ci 49 项通过。原生应用 Connections 仍为 UNVERIFIED。

### Main Changes

- 恢复 OPENAI_CORE_EXACT_DOMAINS，注入 chat.openai.com 及 android/desktop/ios/tcr9i 四个子域
- 修正 openai_core=false DNS 断言，改测 +.host 与 exact 裸键
- quality-guidelines 记录 nameserver-policy 键形态

### Git Commits

| Hash | Message |
|------|---------|
| `babfb35` | (see git log) |
| `b1ba18e` | (see git log) |

### Testing

- [OK] npm run ci（49 通过）
- [OK] 生成规则探测：5 个 exact=1，无 chat.openai.com suffix

### Status

[OK] **Completed**

### Next Steps

- 若可做脱敏 ChatGPT 桌面/iOS Connections，再去掉 UNVERIFIED


## Session 5: v5.8.1 outbound 索引与 UDP 警告汇总

**Date**: 2026-08-17
**Task**: v5.8.1 outbound 索引与 UDP 警告汇总
**Branch**: `dev`

### Summary

为 clash-verge-ai-residential.js 增加一次 main 的 outbound 索引；findOutbound 缺索引即失败。可达叶子 udp:false 改为一条最多 8 样本的汇总警告。版本 5.8.1。npm run ci 59 项通过。boa 5 秒宿主实测仍为 UNVERIFIED。

### Git Commits

| Hash | Message |
|------|---------|
| `5bf50d0` | (see git log) |

### Status

[OK] **Completed**


## Session 6: 拆出 Cursor 仓库索引家宽路由

**Date**: 2026-08-18
**Task**: 拆出 Cursor 仓库索引家宽路由
**Branch**: `dev`

### Summary

将 repo[0-9]+.cursor.sh 从 cursor_core 拆为独立开关 routing.cursor_repository_indexing，默认 false，回落原 Profile。v5.9.0。just ci 通过。Clash Connections 观测 UNVERIFIED。

### Main Changes

- 根脚本新增 ROUTE_CURSOR_REPOSITORY_INDEXING=false，拆分核心与索引正则目录
- allPossibleDomainRegexes 保留两组正则，关闭后可清理 v5.8.1 托管 repo 规则
- 渲染器注册 cursor_repository_indexing，缺字段按 false 补全
- 同步 README、配置文档、路由范围、故障排查与 CHANGELOG
- spec 补充：拆分目录后仍须进入 allPossible* 清理集

### Git Commits

| Hash | Message |
|------|---------|
| `cc714f5` | (see git log) |

### Testing

- [OK] node --test tests/regression.test.js 通过
- [OK] node --test tests/sync-local-config.test.js 通过
- [OK] just ci 通过（62 项测试 + 密钥扫描）
- [OK] 真实 Clash Connections 观测 UNVERIFIED

### Status

[OK] **Completed**

### Next Steps

- 对本机执行 just render-local，使 .local.js 升级到 v5.9.0
- 如需恢复 v5.8.1 repo 家宽路由，设 cursor_repository_indexing=true 后再渲染
- 08-18-residential-monitor-mvp 仍为 planning，未纳入本次提交


## Session 7: 交付家宽监控 C0/C1 并归档

**Date**: 2026-08-18
**Task**: 交付家宽监控 C0/C1 并归档
**Package**: residential-monitor
**Branch**: `dev`

### Summary

建立 residential-monitor 子项目，完成 C0 基础验证与 C1 采集内核，实测 A=50/250/1000 与 10k 30 分钟峰值。

### Main Changes

- 新增 Tauri 2 子项目与 monitor-check / Windows CI 聚合
- C0 binding/凭据/三档 30 天库/10k 峰值证据写入任务目录
- C1 ControllerSession、核算、core schema、幂等 writer、隔离 kill 与 C1 30m replay

### Git Commits

| Hash | Message |
|------|---------|
| `2738e6a` | (see git log) |
| `0edc537` | (see git log) |

### Testing

- [OK] just monitor-check 退出码 0
- [OK] C1 replay 10k/1Hz/30m p95 21ms

### Status

[OK] **Completed**

### Next Steps

- 下一会话启动 08-18-monitor-desktop-realtime（C2）


## Session 8: C2 桌面外壳与实时监控

**Date**: 2026-08-18
**Task**: C2 桌面外壳与实时监控
**Branch**: `dev`

### Summary

交付家宽监控 C2：托盘生命周期、原子订阅、设置向导、Recovery Shell 与 just tdev/tinstall。just monitor-check 与 just ci 退出码 0。未写本机 NSIS/自启动/Credential Manager。已归档 08-18-monitor-desktop-realtime。

### Git Commits

| Hash | Message |
|------|---------|
| `143af87` | (see git log) |

### Status

[OK] **Completed**


## Session 9: C3 历史报告与数据管理

**Date**: 2026-08-18
**Task**: C3 历史报告与数据管理
**Branch**: `dev`

### Summary

交付 ReportService、快照 token、流式导出、精确保留与 Recovery restore。just monitor-check 与 just ci 退出码 0。完整 30 天 A=50/250/1000 重跑未执行。已归档 08-18-monitor-reporting-data，未启动 C4/C5，未归档父任务。

### Git Commits

| Hash | Message |
|------|---------|
| `171650b` | (see git log) |

### Status

[OK] **Completed**
