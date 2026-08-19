# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

在 Windows 11 本机运行 Clash Verge Rev / mihomo 的人。他们打开这个桌面窗口，是为了看清家宽链路上正在发生和已经发生的连接事实，并完成设置、备份、告警和恢复。他们处于操作状态，不是浏览营销页。

## Product Purpose

「家宽流量监控」持续采集控制器上的全部连接事实，按用户重点目标分类，提供实时监控、历史报告、导出、告警、保留和备份恢复。

成功标准：用户能把 controller meter、可归因观测、缺口和未知分开读；不会把观测下界当成账单。

## Positioning

产品只提供观测下界。控制器 meter 与可归因观测总量不得混称为同一全局口径。缺口、未知和能力不支持不得写成零。相邻的代理商控制台或机场面板不能诚实复制这条约束。

## Operating Context

- 本地 Tauri WebView 桌面窗，默认 1200×800，与 Clash Verge Rev 同时运行。
- 数据只留本机。无遥测，无云同步，无应用内自动更新，无 Windows Service。
- TCP secret 只进本机凭据库或当前进程内存。设置页密码框可以回填并切换显示。日志、SQLite、Channel、导出仍不得出现 secret。
- 普通卸载不删除本地库。删除全部本地数据走预览和二次确认短语。
- 数据库无法按普通 schema 打开时，只进入 Recovery Shell。

## Capabilities and Constraints

- 固定五页：概览、实时连接、分析报告、告警、设置 / 数据管理。
- 只能关闭单条当前连接，没有关闭全部入口。
- v1 只支持 Windows 11 NSIS current-user。不发布 macOS / Linux。
- 前端是 Vanilla TypeScript + Vite，不引入 UI 框架，禁止远程 URL 和 CDN。
- 前端只保存视图选择和 DTO 缓存，不在浏览器里重做核算或 Top N。
- 本次全页视觉重做保持专家工具密度：表格、指标和表单仍一屏可扫，不改成大留白营销卡片。

## Brand Commitments

- 产品名：家宽流量监控。
- 界面、注释与子项目文档用中文。
- 绑定口号：观测下界，不是账单。
- 用户已决定：本次重构替换整套视觉世界，而不是打磨现有深色工作台。
- 用户在方向轮选择了品类默认：深色侧栏工作台。按完整工艺执行约定界面，不掺反讽或偷渡的怪癖。质量条对齐 Clash Verge Rev。

## Evidence on Hand

- 真实内容是本机控制器采样、报告快照、告警记录和关于信息。不得编造客户评价、精度承诺或账单数字。
- 仓库内现有界面是功能壳，不是视觉权威。

## Product Principles

- 口径分开：meter、可归因、缺口、未知各自成项。
- 未知保持未知：缺失值显示「未知」，不填零，不填伪默认。
- 本地与最小暴露：secret 与原始流量内容不进界面。
- 操作优先：窗口是工作台，导航为认页服务，不为展示服务。
- 失败可恢复：存储、迁移、通知失败都给出中文下一步，不假装成功。

## Accessibility & Inclusion

已有 skip link、`:focus-visible` 和 `prefers-contrast: more`。未另立 WCAG 等级。界面必须保持键盘可到达侧栏与主操作。
