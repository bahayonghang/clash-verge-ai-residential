# 设置关于页与侧栏宽度：仓库证据

日期：2026-08-20。只记录当前代码与仓库事实，不作为实现清单。

## 截图对照

设置 → 关于。二级导航选中「关于」。主卡标题「关于与发布」，说明「版本、签名状态和固定发布地址。」正文「尚未加载关于信息。」按钮「刷新关于」「显示 GitHub Releases 地址」。卡片纵向拉满工作区。

## 关于数据已存在，默认不加载

| 项 | 位置 |
|---|---|
| `let about: AboutDto \| null = null` | `residential-monitor/src/main.ts:1431` |
| 空态文案 `settings.about_idle` | `src/i18n/zh.ts:360`、`en.ts:360` |
| 点击才 `get_about` | `main.ts:2717-2723` |
| 三段段落渲染 | `main.ts:945-955` |
| `open_releases` 写入 `errorZh` | `main.ts:2732-2738` |
| 命令只返回 URL | `src-tauri/src/lib.rs:906-908` |
| AboutDto 字段 | `src-tauri/src/c5/about.rs:7-36`、`src/dto.ts:342-354` |
| `signed === true` 解码失败 | `src/dto.ts:382-384` |
| 末卡 `min-height: 100%` | `src/styles.css:1550-1553` |

进入关于分区只改 `settingsSection` 再 `paint()`（`main.ts:2248-2271`），没有对应外观分区的 `loadAppearanceFonts`。

## 仓库身份事实

来自 `src-tauri/src/identity.rs` 与 `c5/about.rs`：

- 产品名：家宽流量监控
- binary：residential-monitor
- identifier / AUMID：`io.github.bahayonghang.residential-monitor`
- version：`env!("CARGO_PKG_VERSION")`，当前 Cargo.toml `0.1.0`
- Releases：`https://github.com/bahayonghang/clash-verge-ai-residential/releases`
- signed / updater / Windows Service：均为 false
- 签名说明：本候选未做 Authenticode 签名……

静态、已提交、不随 `get_about` 变化：

- LICENSE：MIT，Copyright (c) 2026 bahayonghang
- README / PRODUCT：Windows 11 NSIS current-user；无遥测；数据只留本机
- 口号「观测下界，不是账单。」已在侧栏 `.brand-slogan`，关于页不必重复
- 日志目录已在数据分区，关于页不必再放路径

`open_log_dir` / 隐私文档写明 WebView 无 opener / fs。发布地址只能展示，不能在应用内打开浏览器。

## 侧栏现状

| 项 | 位置 |
|---|---|
| `.shell { width: 13.75rem; flex: 0 0 13.75rem; }` | `src/styles.css:298-306` |
| 壳 markup | `main.ts:1340-1351` |
| 实时列宽 pointer capture | `main.ts:2099+`、`src/live-table-layout.ts` |
| 外观键 `put_setting` | `src-tauri/src/theme.rs`、`c2/facade.rs` save_ui_* |
| 规范：外观与 `live_table_layout` 不进控制器 JSON | `.trellis/spec/residential-monitor/backend/modules-and-errors.md`、`frontend/view-state.md` |

归档 `08-18-monitor-shell-sidebar` 把固定宽度写成当时需求。本次用户要求可自由改宽度，覆盖该约束。设置二级导航是 `.settings-nav`，产品文案里的「侧边栏」是 `.shell`。
