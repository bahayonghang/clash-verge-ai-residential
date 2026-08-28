# Design：彩色应用图标与安装态清晰度

## 现状与目标

- 现状：Windows 快捷方式使用 `icon.ico` 单层 16×16；产品标记是白瓷砖细线房子。开始菜单 squircle 再套一层圆角，图标又小又糊。
- 目标：C 方案彩色屋顶+柱铺满正方形；多尺寸 ICO 写入 EXE/快捷方式；侧栏与托盘共用同一真源。

## 边界

| 决策点 | 选择 | 理由 |
|---|---|---|
| 视觉 | 方案 C 加粗无烟囱稿 | 用户选定；色块比描边房子更能过 16px |
| 画布 | 不透明、铺满、不预做圆角 | Windows 11 自己套 squircle |
| 真源 | 检入 1024×1024 PNG | JPEG 不可打进 ICO；CI 不调用 Imagine |
| ICO 生成 | 已有 `@tauri-apps/cli icon` | 不新增运行时依赖 |
| 16/24 层 | 先整图下采样，糊了再画简化稿 | 多数情况多尺寸 ICO 已能消除「16 放大」；细节糊才加手工层 |
| 托盘 | `image_edit` 从新 `icon.png` 加右下角色点 | `tray_chrome` 四态映射保持；窗口 `icon.png` 仍无状态点 |
| 侧栏 | `mark-app.png` 同源裁切 | 去掉 JPEG；`sidebar.tsx` 的 `rounded-xl` 继续由 CSS 圆角 |
| Store 磁贴 PNG | 生成后删除 | v1 只发 NSIS，不进 Store |

## 资源流

```
C-roof-barchart-master.jpg  (任务草稿，不入库产品目录)
        │  转 PNG / 必要时再 Imagine 修一版
        ▼
src-tauri/icons/icon-source.png   1024×1024 真源
        │  just monitor-icons  →  npx tauri icon
        ├─ icon.ico   16/24/32/48/64/256
        ├─ 32x32.png, 128x128.png, 128x128@2x.png, icon.png
        └─ （删除 Square* / StoreLogo / icon.icns）
        │
        ├─ src/assets/icons/mark-app.png     侧栏品牌
        └─ tray-collecting/connecting/paused/fault.png
              同源 + 右下角状态点（绿/蓝/琥珀/红 + 海军描边）
```

`tauri.conf.json` `bundle.icon` 指向生成出的 PNG 与 `icon.ico`。窗口 `lib.rs` 继续 `include_image!("icons/icon.png")`。托盘继续 `include_image!("icons/tray-*.png")`。

## ICO 断言

`residential-monitor/scripts/check-icons.mjs` 解析 ICO 目录表：`width==0` 视为 256。断言集合包含 16、32、48、256。挂到 `residential-monitor` 的 `npm run check`。不引入 Pillow/sharp。

## 兼容与回滚

- 不改 `tray_chrome`、产品名、AUMID、NSIS hook。
- 回滚 = 还原 `src-tauri/icons/`、`mark-app.*`、`nav-icons.ts`、`tauri.conf.json`、`justfile`、check 脚本。
- 已安装用户需再跑安装包才会换开始菜单图标；开发态 `just tdev` 即可看到窗口/托盘。

## 风险

- `tauri icon` 覆盖同名 PNG/ICO。配方先备份 `tray-*.png` 或生成到临时目录再拷回需要的文件。
- Windows 图标缓存：安装态验收前重启 Explorer 或改文件名版本，避免误判。
- 托盘 16px 通知区：状态点必须够大；点画在右下角空白渐变上，不要压在柱顶。
