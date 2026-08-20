# 优化设置关于页与侧栏宽度

## Goal

让设置「关于」分区进入后就能读到仓库里真实存在的身份与发布事实，并把应用壳左侧栏改成可拖动、可键盘调整、重启后保持的宽度。用户在 1200×800 窗口里先认页，再读身份，不必先点「刷新关于」，也不必接受写死的 13.75rem 侧栏。

## Background / Confirmed facts

- 用户截图停在设置 → 关于。主内容卡几乎空，文案是「尚未加载关于信息。」，动作是「刷新关于」和「显示 GitHub Releases 地址」。末张 `.settings-card` 使用 `min-height: 100%`（`residential-monitor/src/styles.css:1550-1553`），空内容被纵向拉满。
- `about` 在 `main.ts:1431` 初值为 `null`。只有点击 `#load-about` 才 `invoke get_about`（`main.ts:2717-2723`）。进入关于分区不会自动加载（对照外观分区进入时拉字体列表，`main.ts:1593-1595`）。
- 后端 `c5::AboutDto` 已有：`productName`、`binaryName`、`identifier`、`aumid`、`version`、`releasesUrl`、`signed`、`updaterPlugin`、`windowsService`、`signatureNoteZh`（`src-tauri/src/c5/about.rs:7-36`）。当前候选 `signed=false`，`updaterPlugin=false`，`windowsService=false`。前端 `decodeAbout` 在 `signed === true` 时失败（`src/dto.ts:378-385`）。
- 加载成功后关于区只拼三段 `<p>`（`main.ts:945-955`）。`#open-releases` 调用 `open_releases`，把返回 URL 写进 `state.errorZh`（`main.ts:2732-2738`）。`open_releases` 只返回固定字符串，不打开浏览器（`src-tauri/src/lib.rs:906-908`）。capability 不授予 opener。
- 稳定身份来自 `identity.rs`：产品名「家宽流量监控」，binary `residential-monitor`，identifier / AUMID `io.github.bahayonghang.residential-monitor`，Releases `https://github.com/bahayonghang/clash-verge-ai-residential/releases`。许可证 MIT。README / PRODUCT：v1 只支持 Windows 11 NSIS current-user；无遥测；数据只留本机。
- 应用壳 `.shell` 宽度写死 `13.75rem` / `flex: 0 0 13.75rem`（`styles.css:298-306`）。归档任务 `08-18-monitor-shell-sidebar` 当时要求固定宽度。实时表列宽已有 pointer capture、取消、键盘 separator 和 `put_setting("live_table_layout")` 可复用。
- 外观键 `ui_theme` / `ui_font` / `ui_font_size` / `ui_density` 走 `put_setting`，不进控制器 JSON；Recovery 无库时只改内存。设置页 `connectionDelta` / `summaryChanged` / `alertChanged` 不得整页 `paint`，除非 `errorZh` 变化。
- 本任务保持 planning，直到用户批准本规划摘要后才能 `task.py start`。

## Requirements

### R1. 关于分区展示真实身份

- 进入设置「关于」分区时自动 `get_about` 并解码。会话内缓存，直到用户点「刷新关于」或解码失败后重试。加载中、失败、成功各有可读状态。禁止默认停在「尚未加载关于信息。」
- 用可扫读的标签/值行展示 AboutDto：产品名、版本、可执行文件、identifier、AUMID、签名状态与既有签名说明、无应用内自动更新、无 Windows Service、固定 GitHub Releases URL。URL 在卡片内以可选中等宽文本出现，不得再写入 `errorZh`。
- 另用 i18n 只读行展示仓库已提交的静态事实：MIT 许可证、Windows 11 NSIS current-user、数据只留本机且无遥测。这些不新增 AboutDto 字段，不随运行时变化。
- 保持 `signed === true` 解码失败。不把未签名标成已签名。不编造精度、账单、客户评价、git hash、changelog 或「即将签名」。
- 中英文案齐全。窄窗口标签/值单列。末张卡片继续撑满工作区，内容用定义列表占满空底，不靠装饰填白。

### R2. 应用壳侧栏可调宽度

- 用户可用鼠标拖动 `.shell` 右缘，也可用键盘调整宽度。默认 220px（现 13.75rem @ 16px）。合法范围 160–352px，使 1200×800 下导航文字仍可读、主区仍能工作。
- 宽度写入本机键 `ui_sidebar_width`，整数 CSS 像素，不进控制器 JSON。启动从 `BootstrapDto` 恢复。非法、缺失回落 220。Recovery 无库时只改内存。
- 拖动期间只改侧栏/主区几何，不整页 `paint`。松手成功才持久化一次。pointercancel、失焦、捕获丢失回滚到拖动开始宽度。保存失败保留内存宽度并给出非阻断诊断。
- 五页业务壳与 Recovery 壳共用同一宽度。设置二级导航 `.settings-nav` 不提供独立拖动手柄。外观分区不新增宽度控件。
- 拖动手柄有可见 focus、`role="separator"`、`aria-orientation="vertical"` 和方向键说明。`prefers-reduced-motion` 下取消宽度过渡，拖动仍可用。

### R3. 兼容性

- 不新增顶级 route，不改连接/数据/危险分区语义，不改 secret、删除短语、VACUUM、备份恢复。
- 不注册 updater，不新增 Windows Service，不授予 opener / fs。
- 不引入 UI 框架、远程资源或新 npm 依赖。动态重绘仍按元素 `id` 恢复焦点与选区。

## Out of Scope

- 外观、连接、数据、危险分区的内容重做。
- 设置二级导航可调宽度、汉堡菜单、侧栏折叠成纯图标。
- 在应用内打开 GitHub、复制到系统剪贴板（若需额外 capability）。
- git hash、构建时间、changelog、数据目录（数据分区已有日志路径）。
- 改默认窗口尺寸、托盘、单实例、签名流程或 NSIS。
- 替换整份 `DESIGN.md` 视觉世界。

## Acceptance Criteria

- [ ] AC1：进入关于分区后，不点刷新也能看到解码后的身份行；刷新可重拉；失败有中文/英文下一步且不把 `signed: true` 画成已签名。
- [ ] AC2：关于卡展示 R1 列出的 DTO 字段与三条静态事实；Releases URL 在卡内可选中；`open-releases` 不再把 URL 写入 `errorZh`。
- [ ] AC3：1200×800 与窄窗口下关于卡无大块空底、无水平溢出；四套主题与中英标签完整。
- [ ] AC4：侧栏可拖动与键盘调整；宽度 clamp 到 160–352px；重启后从 `ui_sidebar_width` 恢复；非法值回落 220；Recovery 无库不写盘。
- [ ] AC5：拖动取消不留下半宽；设置页既有 skip-paint 规则仍成立；侧栏拖动不得与实时列表拖动互相污染。
- [ ] AC6：typecheck、lint、相关 unit / Rust 测试、build 通过。关于 `signed` 与删除部分失败的既有断言不被绕过。

## Key Decisions

- 关于页自动加载，保留刷新。对照外观分区拉字体列表。
- AboutDto 不扩展。许可证/平台/隐私用 i18n 静态行，来源是 LICENSE、README、PRODUCT.md。
- 发布地址只展示固定 URL。按钮不再借用错误条。
- 「侧边栏」指应用壳 `.shell`，不是设置二级导航。
- 宽度按 CSS 像素持久化，不随字号 rem 缩放。交互复用实时表列宽的 pointer capture 与 separator 键盘模型。

## Task map

| 子任务 | 目录 | 交付 |
|---|---|---|
| 填充设置关于页身份信息 | `08-20-settings-about-identity` | 自动加载、定义列表、静态产品事实、发布地址离开 errorZh |
| 应用壳侧栏可调宽度 | `08-20-shell-sidebar-resize` | 拖动/键盘、`ui_sidebar_width`、Bootstrap 恢复 |

先做关于页，再做侧栏。两者都改 `main.ts` / `styles.css`，顺序执行避免互相覆盖。父任务在两子任务完成后做集成检查，不直接作为实现入口。

## Planning Status

- artifacts: 本文件、`design.md`、`implement.md`、`research/about-and-sidebar-evidence.md`，以及两个子任务的对应文件。
- 实现等待用户明确批准本规划摘要后，对第一个子任务运行 `task.py start`。
