# 设置页字体与紧凑密度

## Goal

在「外观与语言」分区补齐字体、字号和界面密度，让设置页在 1200×800 下填满工作区而不是只占上半屏。选择后立即预览并写入本机，重启后保持。

## Confirmed facts

- 当前外观分区只有语言分段控件和四套 Catppuccin 主题；截图中卡片下方大片空底。
- `ui_locale` / `ui_theme` 走 `put_setting`，不进控制器 JSON；设置页即时保存；非法值回落默认；Recovery 无库时只改内存。
- `html` 未设 `font-size`，正文是系统默认 16px；`font-family` 写在 Mocha `:root`：`"Segoe UI", "Microsoft YaHei", sans-serif`。等宽栈已是 `"Cascadia Mono", "Sarasa Mono SC", ui-monospace, monospace`。
- 禁止远程 URL / CDN / UI 框架 / 新增 npm 依赖。前端只保存视图选择和 DTO 缓存。
- 上一轮设置重构把「自定义字体」标为范围外；本次用户明确要求加入字体、字号和 compact。

## Requirements

- 外观分区增加三组即时控件：字体、字号、界面密度。控件语言与现有分段 / 选项按钮一致，每项显示当前值。
- 字体只提供本机栈，不下载、不打包字体文件：`system`（默认，Segoe UI + 微软雅黑）、`yahei`（微软雅黑 UI）、`serif`（宋体 / 思源宋体系）、`mono`（现有等宽栈）。选项按钮用对应字体预览名称。
- 字号为离散档：`sm` 14px、`md` 16px（默认）、`lg` 18px。通过 `html` 的 `--ui-font-size` 生效；交互控件 `min-height` 保持 40px。
- 密度为 `comfortable`（默认）和 `compact`。`compact` 只压缩分区、卡片、表格、导航的内边距和行距，不缩小命中区。
- 三项分别写入本机键 `ui_font`、`ui_font_size`、`ui_density`。`BootstrapDto` 带可选字段；缺字段或非法值回落默认。保存失败仍应用本次选择并保留诊断。
- 设置页主区在工作区内纵向撑满；窄窗口无水平溢出。不改顶级五页导航，不改连接 / 数据 / 关于 / 删除语义。
- 中英文案齐全；外观分区副标题覆盖字体与密度。无 `transition: all`、无远程资源。

## Acceptance Criteria

- [ ] AC1：外观分区可切换字体、字号、密度；当前项有选中态；切换后全应用立即预览。
- [ ] AC2：重启后三项从本机设置恢复；非法 / 缺失回落 `system` / `md` / `comfortable`；Recovery 无库时只改内存。
- [ ] AC3：`compact` 下表格、卡片、设置行更紧，按钮与分段控件仍 ≥ 40×40 px。
- [ ] AC4：1200×800 设置页工作区纵向被内容/布局占满，窄窗口无截断或水平溢出。
- [ ] AC5：中英键齐全；typecheck、lint、相关 unit / Rust 测试、build 通过。

## Out of scope

- 扫描本机全部字体、用户自定义 CSS、远程字体、可变字体文件。
- 更多主题、强调色、云同步。
- 改连接表单、备份、retention、VACUUM、关于、删除或实时表列布局语义。
- 把 `DESIGN.md` 整份视觉世界换成另一套。

## Key decisions

- 沿用主题 / 语言的「即时保存 + 分键 put_setting」模式，不做成需确认的表单。
- 字号三档，不用连续滑杆，避免与分段控件语言不一致。
- 密度两档即可覆盖用户点名的 compact，不增加 spacious。

## Planning status

- artifacts: `prd.md` `design.md` `implement.md`
- implementation waits for `task.py start` in the isolated worktree `feat/settings-appearance-prefs`.
