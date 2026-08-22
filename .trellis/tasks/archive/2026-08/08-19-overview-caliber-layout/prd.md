# 概览成对口径布局

## Goal

用户在概览一屏读完分开的口径：每组同时看到上行与下行，分类有表，宽窗不再留下半屏灰底。

## Background

父任务：`08-19-ui-catppuccin-layout`。依赖主题 token 已落地，以便新卡片走语义色。

用户已确认策略：成对口径分组，只用 `LiveOverview` 已有字段，不把实时表搬到概览。

截图证据：8 张等权卡、`auto-fit` 7+1 孤儿、「重点分类」为空仍占整行浅卡、`otherDownload` / `gapDownload` / `overDownload` / `categoryDownload` 未上屏。

## Requirements

- 五组成对口径：控制器 meter、可归因观测、其他连接、未归因 gap、over-attributed。每组上行 + 下行。`null` →「未知」。
- 活跃连接、覆盖、健康同一状态区。
- 重点分类表：名称、上行、下行。键为两类 map 的并集。空态一行「无」。
- 网格固定 3 列（窄窗 2 / 1），禁止 `auto-fit` 孤儿卡。不为填空加营销块或连接预览。
- 不改 DTO 形状，不新增核算。文案跟 `uiLocale`。

## Out of Scope

- 主题持久化（先做的子任务）。
- 实时筛选工具条。
- 概览上的 sparkline / Top N / 实时表预览。

## Acceptance Criteria

- [ ] 成对展示已有上下行字段；缺值「未知」。
- [ ] 分类表含下行；空分类不是整幅空洞浅卡。
- [ ] 默认窗口与更宽窗无 7+1 孤儿卡加大块卡片外灰底。
- [ ] 口径未合并。typecheck / lint / test / build 通过。

## Key Decisions

- 成对口径，不拉 `query_live_connections`。
