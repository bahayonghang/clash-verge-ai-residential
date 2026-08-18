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
