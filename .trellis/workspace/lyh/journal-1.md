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


## Session 10: C4 家宽监控告警与诊断

**Date**: 2026-08-18
**Task**: C4 家宽监控告警与诊断
**Branch**: `dev`

### Summary

实施 C4：AlertEngine、schema=3、同事务 outbox、告警中心与脱敏诊断。just monitor-check 与 just ci 退出码 0。C4-AC7 安装态通知与完整 30 天三档库未执行。已归档 08-18-monitor-alerting-diagnostics。未启动 C5，未归档父任务。

### Git Commits

| Hash | Message |
|------|---------|
| `e83cc5c` | (see git log) |

### Status

[OK] **Completed**


## Session 11: C5 发布硬化与归档

**Date**: 2026-08-18
**Task**: C5 发布硬化与归档
**Branch**: `dev`

### Summary

交付 C5 发布硬化入口、文档与自动门证据；发布结论 no-go。未做安装态、完整 30 天库、24 小时 soak、签名或 GitHub Release。已归档 08-18-monitor-release-hardening。未归档父任务。

### Git Commits

| Hash | Message |
|------|---------|
| `15f960f` | (see git log) |

### Status

[OK] **Completed**


## Session 12: 家宽监控壳层侧栏与控制器探测

**Date**: 2026-08-18
**Task**: 家宽监控壳层侧栏与控制器探测
**Package**: residential-monitor
**Branch**: `dev`

### Summary

把桌面壳改成左侧栏，补本地图标与测试/断开连接，并记录 Impeccable 视觉合同。已归档 08-18-monitor-shell-sidebar。

### Main Changes

- 侧栏壳、导航/窗口图标、test_controller/disconnect_controller

### Git Commits

| Hash | Message |
|------|---------|
| `df0f1d9` | (see git log) |
| `a437fde` | (see git log) |
| `b42f528` | (see git log) |
| `582f9f5` | (see git log) |

### Testing

- [OK] npm --prefix residential-monitor run typecheck/lint/test；cargo test c2_facade_contract_tests；cargo clippy --lib -D warnings

### Status

[OK] **Completed**


## Session 13: 接通实时连接并回填设置密钥

**Date**: 2026-08-19
**Task**: 接通实时连接并回填设置密钥
**Package**: residential-monitor
**Branch**: `dev`

### Summary

接通 1 Hz 采集、Channel 转发与实时连接空态；设置页密钥默认保存并回填圆点。tinstall 改为静默安装。

### Main Changes

- 1 Hz HTTP 采集、Channel 订阅表、query_live_connections 填表与可诊断空态
- 设置页密钥默认写入本机凭据，密码框回填并提供显示按钮
- tinstall 使用 NSIS /S 且安装后不启动应用

### Git Commits

| Hash | Message |
|------|---------|
| `9cd49d8` | (see git log) |
| `eeca0e2` | (see git log) |

### Testing

- [OK] npm --prefix residential-monitor run typecheck/lint/test/build 通过
- [OK] cargo test collector_tick/subscription_forward/pause_keeps_existing 此前通过；收尾时本机 windows crate 缺 Win32 模块未能复跑

### Status

[OK] **Completed**

### Next Steps

- 用 just tdev 对本机 Clash Verge 9097 测实时连接与密钥回填


## Session 14: 实时连接 Clash 列、家宽筛选与中英界面

**Date**: 2026-08-19
**Task**: 实时连接 Clash 列、家宽筛选与中英界面
**Package**: residential-monitor
**Branch**: `dev`

### Summary

设置可切换中英；实时表对齐 Clash 十二列并默认筛选家宽。

### Main Changes

- 设置页保存 ui_locale，切换 WebView、托盘、通知与后端错误文案。
- 实时表按 Clash 列展示，补齐端口/入站/速率，默认只看家宽，支持字段精确/包含 AND 筛选。

### Git Commits

| Hash | Message |
|------|---------|
| `251f680` | (see git log) |

### Testing

- [OK] npm --prefix residential-monitor test（41 项通过）
- [OK] cargo check --offline 通过

### Status

[OK] **Completed**

### Next Steps

- 用 just tdev 对照 Clash Verge 9097 点选语言切换与只看家宽


## Session 15: Catppuccin 主题、成对概览与筛选工具条

**Date**: 2026-08-19
**Task**: Catppuccin 主题、成对概览与筛选工具条
**Branch**: `dev`

### Summary

落地四口味主题、概览成对口径和实时筛选工具条。just ci 通过后提交并归档父任务与三个子任务。

### Main Changes

- Catppuccin Latte/Frappé/Macchiato/Mocha 整窗换肤，键 ui_theme
- 概览 3x2 成对上下行与分类表
- 实时筛选横向工具条，查询语义不变

### Git Commits

| Hash | Message |
|------|---------|
| `e5d693a` | (see git log) |
| `0e2ad49` | (see git log) |

### Testing

- [OK] just ci 通过：前端 45、Rust 164 passed / 1 ignored、根回归 62

### Status

[OK] **Completed**

### Next Steps

- 未 push；如需发布再开 PR


## Session 16: 实时表排序筛选与列布局

**Date**: 2026-08-19
**Task**: 实时表排序筛选与列布局
**Branch**: `dev`

### Summary

交付实时连接表固定列宽、拖动显隐、表头排序与数值条件；已提交并归档 08-19-live-table-sort-width 及三个子任务。

### Git Commits

| Hash | Message |
|------|---------|
| `d3d6c7a` | (see git log) |
| `5097bcf` | (see git log) |

### Status

[OK] **Completed**


## Session 17: 分析报告自动小时与日档案

**Date**: 2026-08-19
**Task**: 分析报告自动小时与日档案
**Branch**: `dev`

### Summary

落地闭合本地小时/日冻结 ReportResult 档案、v4 report_archive、采集后分批补跑，以及分析报告页进页读最新档案。手动查询不覆盖档案。just monitor-check 通过。

### Git Commits

| Hash | Message |
|------|---------|
| `4953502` | (see git log) |

### Status

[OK] **Completed**


## Session 18: 分析报告页档案后置与结果可视化

**Date**: 2026-08-19
**Task**: 分析报告页档案后置与结果可视化
**Package**: residential-monitor
**Branch**: `dev`

### Summary

分析报告页将自动档案移到页尾并缩短，总量/趋势/Top N 先行；添加本地 SVG 趋势图与按下行之和对齐的扇形图。typecheck/lint/test/build 通过。

### Main Changes

- 档案后置、约 8 行滚动、类型筛选
- 趋势折线+表，Top N 扇形图分母 totals.download，正差额显示其余

### Git Commits

| Hash | Message |
|------|---------|
| `b51d84d` | (see git log) |

### Testing

- [OK] npm --prefix residential-monitor run typecheck lint test build
- [OK] Vite 空态与构图预览

### Status

[OK] **Completed**


## Session 19: 家宽监控本机日志与 tinstall 停进程

**Date**: 2026-08-19
**Task**: 家宽监控本机日志与 tinstall 停进程
**Package**: residential-monitor
**Branch**: `dev`

### Summary

落地本机文件日志（轮转脱敏、设置页与 Recovery 打开目录、删除清单纳入日志），并让 just tinstall 在安装前结束运行中的 residential-monitor。

### Main Changes

- app_log + redact：LocalAppData logs、轮转、禁止子串扫描
- 设置页与 Recovery 打开日志目录；删除清单含 logs
- just tinstall 安装前 Stop-Process residential-monitor

### Git Commits

| Hash | Message |
|------|---------|
| `0b0295e` | (see git log) |
| `c325be1` | (see git log) |

### Testing

- [OK] cargo fmt/clippy/test workspace
- [OK] npm --prefix residential-monitor typecheck/lint/test/build
- [OK] npm run check:secrets
- [OK] just --show tinstall

### Status

[OK] **Completed**

### Next Steps

- just tdev 走查打开日志目录；未跑 tinstall


## Session 20: 自动连接与设置界面现代化

**Date**: 2026-08-20
**Task**: 自动连接与设置界面现代化
**Branch**: `dev`

### Summary

完成 residential-monitor 启动自动连接与监控恢复、SkillPort 信息架构参考的现代化设置界面、前后端验证与 code-spec 更新；已归档 auto-connect、settings-redesign 及父任务。真实 Tauri WebView 截图和安装态控制器验证仍按任务记录为 UNVERIFIED。

### Git Commits

| Hash | Message |
|------|---------|
| `51d869f` | (see git log) |

### Status

[OK] **Completed**


## Session 21: 实时连接筛选、列宽与热点摘要

**Date**: 2026-08-20
**Task**: 实时连接筛选、列宽与热点摘要
**Branch**: `dev`

### Summary

完成实时连接界面优化：draft/applied 筛选、colgroup 像素列宽、后端完整 matched 集合热点摘要；fail-closed 隐藏暂停/缺口旧值。前端 check 99 tests、cargo test workspace、Vite 1200x800/800 预览与 axe 0 violations。真实 C2/Tauri WebView 未验证。

### Git Commits

| Hash | Message |
|------|---------|
| `7bc91ac` | (see git log) |
| `d1777e8` | (see git log) |

### Status

[OK] **Completed**


## Session 22: 设置页字体、字号与紧凑密度

**Date**: 2026-08-20
**Task**: 设置页字体、字号与紧凑密度
**Package**: residential-monitor
**Branch**: `dev`

### Summary

在外观与语言分区加入本机字体栈、三档字号和 compact 密度，即时预览并写入 ui_font / ui_font_size / ui_density；忽略 .worktrees。

### Main Changes

- 外观分区增加字体、字号、密度控件，选择后立即 apply 并 put_setting
- BootstrapDto 增加 uiFont / uiFontSize / uiDensity，非法值回落 system/md/comfortable
- compact 压缩留白，控件 min-height 保持 40px；.gitignore 忽略 .worktrees/

### Git Commits

| Hash | Message |
|------|---------|
| `d05b5bc` | (see git log) |
| `690131d` | (see git log) |

### Testing

- [OK] npm --prefix residential-monitor typecheck/lint/test/build
- [OK] cargo test --lib ui_font_size_and_density / font_size_and_density_fall_back
- [OK] Vite 预览：中英外观、serif+lg+compact、420px 无水平溢出

### Status

[OK] **Completed**

### Next Steps

- 真实 Tauri WebView 重启后读回三项设置仍为 UNVERIFIED


## Session 23: 分析报告页探查与排版

**Date**: 2026-08-20
**Task**: 分析报告页探查与排版
**Package**: residential-monitor
**Branch**: `dev`

### Summary

修复分析报告页被 Channel 增量冲掉滚动和 details 的问题，补齐图表探查，并收紧该页排版。

### Main Changes

- reports 页跳过无关键 paint，必要重绘写回滚动与 details
- 扇形图与趋势柱可悬停、钉住，并高亮对应表行
- 总量并入结果区，扇形图作为 Top N 色例

### Git Commits

| Hash | Message |
|------|---------|
| `353e910` | (see git log) |

### Testing

- [OK] npm --prefix residential-monitor run typecheck/lint/test/build 通过

### Status

[OK] **Completed**

### Next Steps

- 实机采集运行时核 Top N 滚动、details 展开和钉住探查


## Session 24: 设置页系统字体与工作区排版

**Date**: 2026-08-20
**Task**: 设置页系统字体与工作区排版
**Package**: residential-monitor
**Branch**: `feat/settings-system-fonts-layout`

### Summary

外观字体改为本机可搜索下拉，设置工作区在 1200x800 下占满主区；已归档 08-20-settings-system-fonts-layout。

### Main Changes

- ui_font 从四档枚举改为 system/旧别名/校验族名，新增 list_ui_fonts（GDI）
- 设置工作区 grid-template-rows 1fr，末张卡片 min-height 100%

### Git Commits

| Hash | Message |
|------|---------|
| `aeb778f` | (see git log) |
| `94fc17f` | (see git log) |

### Testing

- [OK] npm typecheck/lint/test/build；cargo fmt/clippy；cargo test theme:: ui_font；Vite 1200x800 与 420 宽预览。Tauri 完整字体列表未实拍。

### Status

[OK] **Completed**

### Next Steps

- 在 just tdev 的 WebView 里核对本机字体列表、搜索与重启恢复


## Session 25: 修复设置页字体选择器重绘打断

**Date**: 2026-08-20
**Task**: 修复设置页字体选择器重绘打断
**Branch**: `feat/settings-system-fonts-layout`

### Summary

规划按审阅结论修订后实施：字体列表补 class 与滚动标记；settings-data 跳过 connectionDelta/summaryChanged/alertChanged；重绘恢复选区与滚动；进入连接分区补拉 tray。just ci 通过。Vite 预览核对 class 与 sm/md 限高。采集运行、输入法与长列表滚动待实机验收。

### Git Commits

| Hash | Message |
|------|---------|
| `b32e605` | (see git log) |

### Status

[OK] **Completed**


## Session 26: 填充关于页身份信息并支持侧栏调宽

**Date**: 2026-08-20
**Task**: 填充关于页身份信息并支持侧栏调宽
**Package**: residential-monitor
**Branch**: `dev`

### Summary

进入关于分区自动加载仓库身份与发布事实；应用壳侧栏可拖动或键盘调整宽度并写入本机。

### Main Changes

- 进入关于自动 get_about，定义列表展示 AboutDto 与 MIT/平台/隐私静态行
- 发布地址留在卡片内，open-releases 不再写入 errorZh
- 侧栏 160-352px 可拖动/键盘调整，键 ui_sidebar_width，默认 220

### Git Commits

| Hash | Message |
|------|---------|
| `aa35817` | (see git log) |

### Testing

- [OK] npm --prefix residential-monitor run typecheck/lint/test/build 通过
- [OK] cargo test --lib --offline --locked 通过
- [OK] Vite 预览核对关于失败态与默认 220px 侧栏；真实 Tauri WebView 未实机核对

### Status

[OK] **Completed**

### Next Steps

- 在 just tdev 的 WebView 里核对关于自动加载、卡内 Releases URL 与侧栏拖动/重启恢复


## Session 27: neko UI 重构实施与收口

**Date**: 2026-08-21
**Task**: neko UI 重构实施与收口
**Package**: residential-monitor
**Branch**: `refactor/neko-ui-port`

### Summary

在 refactor/neko-ui-port 上完成 08-21-neko-ui-refactor 及其六个子任务：React 壳、C3 维度查询、十段页面移植、家宽独立页与父任务收口。just monitor-check 通过。未测 GUI 实拍与 monitor-bench 五维体积。

### Main Changes

- 入口改为 React 19 + Tailwind v4，删除 main.ts 与 styles.css
- C3 增加分钟粒度、last_chain_hop、filters、五维物化与 __unknown__ 排名行
- 十段路由与 residential_share 落地，家宽双口径收敛到一个模块

### Git Commits

| Hash | Message |
|------|---------|
| `718e8d7` | (see git log) |
| `3f0a856` | (see git log) |

### Testing

- [OK] just monitor-check；前端 178 测、cargo test 259 通过

### Status

[OK] **Completed**

### Next Steps

- 在 Tauri 窗口做四主题实拍与能力降级检查
- 补测 monitor-bench 五维物化体积后一次性合入 main


## Session 28: 未知主机归因与英文侧栏

**Date**: 2026-08-21
**Task**: 未知主机归因与英文侧栏
**Package**: residential-monitor
**Branch**: `refactor/neko-ui-port`

### Summary

英文侧栏两行品牌锁与单行导航。空 host 按 sniffHost 或目的 IP 归因，未知行可检查组成。桌面构建去掉 cdylib 与 500kB 警告。

### Main Changes

- 英文侧栏 220px 两行品牌与短口号
- host identity: host then sniffHost then destination IP
- 主机页未知行可下钻；条形图轴标签不再左侧裁切

### Git Commits

| Hash | Message |
|------|---------|
| `c65daa4` | (see git log) |
| `b0dee12` | (see git log) |
| `9dffefe` | (see git log) |
| `db8d770` | (see git log) |

### Testing

- [OK] cargo test --workspace 267 passed; vitest 187; typecheck lint build; clippy -D warnings

### Status

[OK] **Completed**

### Next Steps

- just tdev 实拍英文侧栏与主机页


## Session 29: 修复 Unknown 流量归因

**Date**: 2026-08-21
**Task**: 修复 Unknown 流量归因
**Branch**: `refactor/neko-ui-port`

### Summary

修复 durable writer/controller epoch、canonical metadata 与 DIRECT Chain 误归因；增加 attributionQuality、实时观测阶段和双数据面 UI；独立检查修复瞬时失败保真、并发 epoch、严格帧解码与重物化守恒，完整门禁通过。

### Git Commits

| Hash | Message |
|------|---------|
| `8146e40` | (see git log) |
| `3013093` | (see git log) |
| `cd57f28` | (see git log) |

### Status

[OK] **Completed**
