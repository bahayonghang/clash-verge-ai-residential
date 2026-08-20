# 设置页系统字体与工作区排版

## Goal

操作员在「外观与语言」里从本机已安装字体中选择界面字体。设置工作区在默认 1200×800 窗里纵向占满主区。选择后立即预览并写入本机，重启后保持。

## Background

截图 Image #1：外观分区字体行是 2×2 按钮（系统默认 / 微软雅黑 / 宋体 / 等宽）。同一帧里，二级导航和外观卡片只占主区上半，卡片四周和下方露出 `--main` 空底。

上一轮 `08-20-settings-appearance-prefs` 把「扫描本机全部字体」标为范围外，并要求 1200×800 工作区被占满。实现只给 `.settings-page` / `.settings-layout` 加了 flex；`.settings-layout` 仍是 `align-items: start`，`.settings-card` 按内容收缩。

## Confirmed facts

- `ui_font` 现为四值枚举 `system` / `yahei` / `serif` / `mono`，默认 `system`。权威在 `theme.rs` `UiFont`；前端 `parseUiFont` 把未知值打回 `system`。应用面是 `html[data-font]` 加 `--ui-font` 栈。
- 写入走 `save_ui_font` → `put_setting("ui_font")`，不进控制器 JSON。`BootstrapDto.uiFont` 可选。Recovery 无库时只改内存。写失败仍应用本次选择。
- 字体控件复用 `.theme-grid` 四格，选项名用对应 `font-family` 预览。
- 字号 `sm|md|lg` 与密度 `comfortable|compact` 保持现有档位。
- `#view:has(.settings-page)` 已纵向 flex。空底来自内容卡片 shrink-wrap，不是 `#view` 没长高。
- 禁止远程 URL / CDN / UI 框架 / 新增 npm 依赖。v1 只发 Windows 11。`windows-sys` 已在 Windows 目标依赖中，现有 feature 不含 GDI。
- 本任务留在既有 Catppuccin 工作台视觉世界内，不替换 `DESIGN.md`。

## Requirements

- R1：外观「字体」改为应用内可搜索下拉。首项「系统默认」，对应现有 `system` 栈。其余选项为本机字体族，选项文本用该族预览。选中后整窗立即换字体并写入 `ui_font`。
- R2：列表来自本机已安装字体族，不下载、不打包、不走 CDN、不依赖 WebView `queryLocalFonts`。列表失败时仍可设回系统默认，并给出中文诊断。
- R3：`ui_font` 合法值是 `system`、旧别名 `yahei` / `serif` / `mono`，或一条通过校验的族名。旧别名继续映射原栈。非法、空、含 CSS 元字符的值回落 `system`。
- R4：应用面用 `--ui-font` 加通用回退。未校验字符串不得拼进 stylesheet 或 `data-*`。字体未再安装时保留已存名称，画面回退通用族，选择器仍显示该名称。
- R5：设置工作区（页头 + 二级导航 + 内容面）在 `#view` 内纵向占满。内容列最后一张卡片随列拉高，空底留在卡片内部。五个分组共用该骨架。
- R6：字体控件改为单行选择器。主题四格保留。选项行节奏收紧，命中区仍 ≥ 40×40 px。不堆装饰，不把五个分组拼成一页长滚。
- R7：窄窗口无水平溢出；二级导航既有横向滚动保留。中英文案齐全。无 `transition: all`、无远程资源。
- R8：字号、密度、主题、语言、连接表单草稿、secret 显隐、删除确认语义不变。

## Acceptance Criteria

- [ ] AC1（R1）：外观字体可从可搜索下拉选择本机族；「系统默认」始终在列表顶部；选中后面板、表格、导航立即换字体。
- [ ] AC2（R3, R4）：重启后恢复所选族或 `system`；`yahei` / `serif` / `mono` 仍映射原栈；非法值回落 `system`；Recovery 无库时只改内存。
- [ ] AC3（R2）：列表失败时界面仍能设回系统默认，并显示中文失败说明，不出现空选择器假成功。
- [ ] AC4（R5, R6, R7）：1200×800 下设置页主区被页头、二级导航和拉高的内容面占满；窄窗口无截断或水平溢出。五个分组都走同一工作区骨架。
- [ ] AC5（R8）：字号三档、密度两档、主题四套、语言分段行为与上一轮一致。连接表单草稿、secret 显隐、删除确认语义不变。
- [ ] AC6（R7）：中英键齐全且集合相等；typecheck、lint、相关 unit / Rust 测试、build 通过。

## Out of scope

- 远程字体、可变字体文件、用户自定义 CSS、字重/斜体轴、逐控件字体。
- 改字号档位或密度档位，或增加 spacious。
- 更多主题、强调色、云同步。
- 改连接、备份、retention、VACUUM、关于、删除的后端语义。
- 替换 `DESIGN.md` 视觉世界。
- 改概览 / 实时 / 报告 / 告警页布局。
- 运行中安装新字体后的热刷新。

## Key decisions

- 选择器：应用内可搜索下拉（2026-08-20 用户确认），不用 Windows 字体对话框，不用无搜索的原生 `<select>`。
- 保存：沿用主题路径，即时 `put_setting`，不做需确认的外观表单。
- 布局：先改工作区骨架（内容列与末张卡片纵向 stretch），不靠堆装饰填空。
- 列表：Windows GDI 枚举族名；跳过 `@` 竖排面和空名；其余族可搜。
- 取值：`system` 为哨兵；旧四档别名兼容；新值是校验后的族名。

## Planning status

- artifacts: `prd.md` `design.md` `implement.md`；`implement.jsonl` / `check.jsonl` 已写入 spec 条目
- 阻塞项：无
- implementation waits for the user to approve this planning summary, then `task.py start`
