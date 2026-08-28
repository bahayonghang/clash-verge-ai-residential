# 彩色应用图标与安装态清晰度

## 目标与用户价值

Windows 开始菜单、任务栏和桌面快捷方式上的「家宽流量监控」图标目前是灰白房子，安装后发糊。换成彩色「屋顶 + 柱状图」产品标记，并让安装态各尺寸都清晰可读。

## 已确认事实

1. 用户截图是 Windows 开始菜单/搜索里的产品快捷方式：灰白圆角房子 + 标题「家宽流量监控」。
2. `residential-monitor/src-tauri/icons/icon.ico` 只有 **16×16** 一层，文件 748 字节。Windows 把这一层放大后发糊。
3. 同目录已有 `32x32.png`、`128x128.png`、`icon.png`（512×512），`tauri.conf.json` 的 `bundle.icon` 同时列出这份 16×16 `icon.ico`。Windows 快捷方式读 ICO/EXE 资源。
4. 源标记是嵌套圆角白瓷砖 + 细线房子和 wifi（`icon.png`、`src/assets/icons/mark-app.jpg`）。Windows 11 再套 squircle，开始菜单里房子被缩进、发灰。
5. 侧栏 `BRAND_MARK` 来自 `mark-app.jpg`（JPEG），56×56 显示（`sidebar.tsx`）。托盘四态 `tray-*.png` 从同一白瓷砖房子派生；窗口图标 `include_image!("icons/icon.png")` 不随托盘状态变。
6. 仓库没有 ICO 生成配方。`justfile` 无 icon 目标。`installer.nsh` 不处理快捷方式图标。
7. `DESIGN.md` 图表色：`#3b82f6` `#8b5cf6` `#06b6d4` `#10b981` `#f59e0b`。产品名与安装名保持「家宽流量监控」。v1 只发 Windows 11 NSIS current-user。
8. 用户选定方案 C：蓝底 + 屋顶 + 彩色柱。实施真源为 `research/candidates/C-roof-barchart-master.jpg`（去掉烟囱、加粗色块）。

## 需求

- R1 以选定的 C 标记为唯一视觉真源。画布铺满 1:1，不预做圆角蒙版，不写字。Imagine JPEG 只作草稿，入库真源必须是无损 PNG。
- R2 从真源导出多尺寸 PNG，并打出含 16 / 24 / 32 / 48 / 64 / 256 的 `icon.ico`。256 层用 PNG 压缩。禁止继续提交单层 16×16 ICO。
- R3 更新 `bundle.icon`、窗口 `icon.png`、侧栏品牌图（改为 PNG）、托盘四态，全部派生自同一真源。托盘仍用绿 / 蓝 / 琥珀 / 红状态点 + 深色描边区分四态；`tray_chrome` 映射不变。
- R4 增加可重复的图标生成步骤（`just` 配方调用已有 `@tauri-apps/cli icon`），避免手改 ICO。
- R5 安装后再看开始菜单、任务栏、桌面快捷方式、窗口标题栏和托盘：彩色、边缘清楚。
- R6 16×32 预览若柱与屋顶糊成一团，为 16 / 24 层单独画简化稿（屋顶 + 三根粗柱），再打进同一 ICO。

## 验收标准

- A1 `icon.ico` 至少包含 16、32、48、256 四层；256 为 PNG 压缩。由仓库脚本解析 ICO 目录表断言，不靠目测文件大小。
- A2 `bundle.icon` 列出与生成产物一致的 PNG/ICO。
- A3 侧栏品牌图是 PNG，`nav-icons.ts` 不再引用 `mark-app.jpg`。
- A4 托盘四态仍由 `tray_chrome` 区分，glyph 与产品标记同一套；状态点在右下角，不改会话映射。
- A5 开发态 `just tdev` 窗口与托盘已是新标记。安装态（`just tinstall`，需用户再确认后才执行）开始菜单与任务栏清晰、彩色。
- A6 真源与派生文件旁的 `*.json` 记录 prompt / 派生关系，继续本地生成、不走 CDN。
- A7 `just monitor-check` 覆盖 ICO 层数断言。

## Out of Scope

- 概览/报告页里的 Recharts 业务图表。
- 十段导航的 route 图标（`overview.jpg` 等）。
- macOS / Linux 图标与 Microsoft Store 磁贴资源（`tauri icon` 若写出 StoreLogo，提交前删除）。
- 产品名、AUMID、安装/卸载契约。
- 未再确认前执行 `just tinstall`。

## 风险

- `tauri icon` 会覆盖 `src-tauri/icons/` 下同名文件；托盘四态文件名不在默认清单里，生成后需确认 `tray-*.png` 仍在。
- 16px 下采样可能把五根柱糊在一起；R6 是缓解，不是另开任务。
- 安装态图标受 Windows 图标缓存影响；验收时若仍显示旧图，需刷新 Explorer 缓存后再判断。
