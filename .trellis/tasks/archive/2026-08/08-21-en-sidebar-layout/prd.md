# 英文侧栏品牌与底栏排版

## Goal

在默认侧栏宽度 220px 下，英文界面的左上产品锁与左下设置项按设计换行。中文界面保持单行可扫读。

## Background

父任务：`08-21-unknown-host-en-sidebar`。截图：English、Sidebar width 220、Density Compact、Font size Medium。

`sidebar.tsx` 品牌区 `h1.text-xl.leading-tight` 直接渲染 `product.display_name`；口号 `min-h-8 text-xs`；导航与底栏无 `nowrap`。英文 `Residential Traffic Monitor`、`Live connections`、`Settings / data` 被挤成单词级换行。官方显示名保持 `Residential Traffic Monitor`。宽度契约不变：默认 220，范围 160–352。

已确认：不靠缩短导航英文、不靠用户拖宽侧栏；用锁的两行结构、口号专用句和单行截断来适配 220px。

## Requirements

- 英文品牌锁两行：`Residential` / `Traffic Monitor`。中文「家宽流量监控」一行。关于弹层与设置页仍用完整 `product.display_name`。
- 侧栏口号最多三行、按词断开。英文侧栏专用句：`Observed lower bound, not a bill.` 设置页与关于弹层仍用完整口号。
- 导航与底栏：图标 `shrink-0`，标签单行；220px compact Medium 下 `Live connections` 与 `Settings / data` 完整可见。160px 下允许 `truncate`，图标仍在。
- 选中态仍是整块主色圆角条。
- 四主题、中/英、两种密度、三档字号下，品牌区与底栏都不从单词中间切断。

## Out of scope

- 未知主机归因（`08-21-unknown-host-attribution`）。
- 改侧栏宽度契约或持久化命令。
- 改主区设置页表单布局。
- 仅图标导航。

## Acceptance Criteria

- [ ] AC1：英文、220px、compact、Medium：品牌区为设计两行；口号按词换行且不超过三行。
- [ ] AC2：同条件下 `Live connections` 与 `Settings / data` 单行完整；图标与文字垂直居中。
- [ ] AC3：中文、220px：产品名单行；「设置 / 数据管理」可扫读。
- [ ] AC4：160px 下英文标签可截断并保留图标；底栏不被顶出视口。
- [ ] AC5：`npm --prefix residential-monitor` typecheck / lint / test / build 通过；侧栏测试覆盖中英结构。
