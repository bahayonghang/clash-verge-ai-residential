# Implement：彩色应用图标与安装态清晰度

## 顺序清单

1. 把 `research/candidates/C-roof-barchart-master.jpg` 转成 1024×1024 无损 PNG，写入 `residential-monitor/src-tauri/icons/icon-source.png`，旁挂 `icon-source.png.json`（prompt + 选定方案 C）。
2. 新增 `just monitor-icons`：调用 `npx tauri icon icon-source.png`，只保留 `icon.ico`、`icon.png`、`32x32.png`、`128x128.png`、`128x128@2x.png`；删除 StoreLogo / Square* / icns；确认 `tray-*.png` 未被覆盖。
3. 新增 `residential-monitor/scripts/check-icons.mjs`：解析 ICO 目录表，断言含 16/32/48/256。挂到 `package.json` 的 `check`。
4. 更新 `tauri.conf.json` `bundle.icon` 为生成清单。
5. 用 Imagine `image_edit` 从新 `icon.png` 派生托盘四态（绿/蓝/琥珀/红点 + 海军描边，右下角），更新对应 `*.json`。
6. 侧栏：新增 `src/assets/icons/mark-app.png`（同源），改 `nav-icons.ts`；删除 `mark-app.jpg`。
7. 16/32 预览：若柱糊掉，为 16/24 画简化层并写回 ICO，再跑 check-icons。
8. 验证：
   - `npm --prefix residential-monitor run check`
   - `just monitor-check`
   - `just tdev` 看窗口标题栏与托盘
   - `just tinstall` 仅在用户本轮确认后执行，看开始菜单/任务栏；必要时刷新 Explorer 图标缓存

## 验证命令

```bash
just monitor-icons
npm --prefix residential-monitor run check
just monitor-check
just tdev
```

安装态（需再确认）：

```bash
just tinstall
```

## 风险文件与回滚点

- `residential-monitor/src-tauri/icons/*`：整目录可还原。
- `residential-monitor/src/nav-icons.ts`：一行 import。
- `residential-monitor/src-tauri/tauri.conf.json`：`bundle.icon` 数组。
- `justfile` / `package.json`：新增配方与 check 脚本入口。
- 不改 `lib.rs` 托盘映射，除非 `include_image` 路径改名。

## task.py start 前检查

- [x] `prd.md` 已过收敛：无未决 Open Questions。
- [x] `design.md` / `implement.md` 就绪。
- [x] `implement.jsonl` / `check.jsonl` 已含真实 spec/research 条目。
- [ ] 用户明确批准本规划摘要后才 `task.py start`。
