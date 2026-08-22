# 应用壳侧栏可调宽度

## Goal

用户可以拖动或用键盘改变应用壳左侧栏宽度，并在重启后保持。默认宽度与现在视觉接近，主区在 1200×800 下仍能工作。

## Dependencies and confirmed facts

- 父任务：`08-20-settings-about-sidebar`。本子任务在关于页子任务之后实施，避免同时改 `main.ts` / `styles.css`。
- `.shell` 现为固定 `13.75rem`（`src/styles.css:298-306`）。壳 markup 在 `main.ts:1340-1351`。
- 可复用实时表列宽：pointer capture、cancel、`role="separator"`、`put_setting`。外观键模式见 `save_ui_theme` / `ui_density`（`c2/facade.rs`、`theme.rs`）。
- 规范：视图宽度类偏好走本机设置键，不进控制器 JSON；Recovery 无库只改内存（`view-state.md`、`modules-and-errors.md`）。
- 设置二级导航 `.settings-nav` 不是本需求中的侧栏。

## Requirements

- `.shell` 右缘提供拖动手柄。鼠标拖动与键盘（左右箭头步进 8px，Home 最小，End 最大）都可改宽度。
- 默认 220px，范围 160–352px。使用 CSS 像素，不随字号 rem 缩放。
- 键 `ui_sidebar_width` 经 `put_setting`。`BootstrapDto.uiSidebarWidth` 可选。非法/缺失回落 220。Recovery 无库只改内存。
- 拖动期间禁止整页 `paint`，只改壳几何。松手且有变化才保存一次。cancel / lostpointercapture / 失焦回滚到拖动开始值。保存失败保留内存宽度并给非阻断诊断。
- 五页与 Recovery 共用该宽度。不改设置二级导航、不加外观分区控件、不引入折叠成纯图标。
- 手柄可见 focus、`aria-orientation="vertical"`、`aria-valuemin/max/now`。reduced-motion 下无宽度过渡。与实时列宽 dragging 互斥。

## Acceptance Criteria

- [ ] AC1：拖动与键盘可把侧栏宽度限制在 160–352px；默认 220。
- [ ] AC2：重启后从 `ui_sidebar_width` 恢复；非法值回落 220；Recovery 无库不写盘。
- [ ] AC3：拖动取消不留下半宽；与实时列表拖动互不污染；设置 skip-paint 仍成立。
- [ ] AC4：1200×800 主区仍可用；窄窗口无水平溢出；中英 aria 文案齐全。
- [ ] AC5：相关 TS parse/clamp 测试与 Rust persist/回落测试通过；typecheck、lint、build 通过。

## Out of scope

- 设置二级导航宽度、汉堡菜单、纯图标折叠。
- 改默认窗口尺寸或导航信息架构。
- 关于页内容（由兄弟子任务负责）。
