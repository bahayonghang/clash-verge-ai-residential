# 设计：本机字体下拉与设置工作区

Visitor mode: Operate。设置页是任务面，不是展示面。

## Shape brief

- Job：在本机字体中选出界面字体，并让设置工作区占满主区。
- Audience：Windows 11 上同时开着 Clash Verge Rev 的操作员。
- Outcome：下拉能搜到已安装族；「系统默认」始终可回；1200×800 下卡片不再漂在 `--main` 空底上。
- Direction：保留 Catppuccin 工作台。字体从四格按钮改成单行 combobox。内容列末张卡片 stretch 填满工作区。
- Untouched：字号、密度、主题四格、连接/数据/关于/删除语义、其他四页。

## Boundaries

- Rust 仍是 `ui_font` 权威：解析、回落、`put_setting`。前端只把已通过校验的字符串写到 `--ui-font`。
- 不进控制器 JSON。不新增 SQLite 表或 schema version。`machine_setting.ui_font` 从四档枚举变为字符串。
- 不新增 npm 依赖。`windows-sys` 增加 `Win32_Graphics_Gdi`（及枚举 HDC 所需的现有 Foundation）。
- 字体列表不进 `BootstrapDto`。独立命令 `list_ui_fonts`，结果只留当前会话视图缓存。

## Contracts

### `ui_font` 值

| 存值 | 含义 | `--ui-font` |
|---|---|---|
| `system`（默认） | 产品栈 | `"Segoe UI", "Microsoft YaHei", sans-serif` |
| `yahei` | 旧别名 | `"Microsoft YaHei UI", "Microsoft YaHei", sans-serif` |
| `serif` | 旧别名 | `"Source Han Serif SC", "Noto Serif CJK SC", "Songti SC", SimSun, serif` |
| `mono` | 旧别名 | `"Cascadia Mono", "Sarasa Mono SC", ui-monospace, monospace` |
| 其它 | 本机族名 | `"<name>", sans-serif` |

`UiFont` 从 Copy 枚举改为透明 newtype `UiFont(String)`。`parse`：

1. 缺省 / 空白 → `system`
2. `system` / `yahei` / `serif` / `mono` → 原样
3. 通过族名校验 → trim 后原样保存
4. 其余 → `system`

族名校验（前后端同一规则）：

- 非空，UTF-16 码元数 ≤ 31（`LF_FACESIZE - 1`）
- 不以 `@` 开头
- 不含控制字符和 `" ' ; { } < > \\`

应用：`applyFont` 对 `html` `setProperty("--ui-font", stack)`。自定义族设 `data-font="custom"`，避免把空格族名放进 `data-*`。`save_ui_font` 仍返回规范化字符串。

### `list_ui_fonts`

```
list_ui_fonts() -> Result<Vec<String>, AppErrorDto>
```

- Windows：`EnumFontFamiliesExW`，`DEFAULT_CHARSET`，HDC 用显示设备。
- 收集 `LOGFONTW.lfFaceName`，跳过空名和 `@` 竖排面，大小写不敏感去重，按不区分大小写排序。
- 失败：`code=io`，`error.font_list`，`action.retry`，`retryable=true`。
- 非 Windows 编译桩返回空向量（产品不发那些目标）。
- 不写入设置、不碰 facade 锁之外的采集状态；命令本身可不持 `AppFacade` 锁。

前端会话缓存：`uiFontFamilies: string[]`、`uiFontListError: string`、`uiFontListLoaded: boolean`。首次绘制外观分区且未加载成功时 invoke 一次，再 `paint` 一次。筛选键不 `paint`。

## Data flow

```
外观首次绘制 → list_ui_fonts → 缓存族名 → 再绘下拉
下拉选中     → parseUiFont → save_ui_font → put_setting
             ↘ 失败仍 applyFont（与主题相同）
boot         → BootstrapDto.uiFont → parseUiFont → applyFont
```

下拉选项 = `system` +（当前值若不在列表中则插入）+ 缓存族名。当前值是旧别名时显示中英标签，族名显示族名。

Combobox：触发钮 `aria-haspopup="listbox"` `aria-expanded`，面板内筛选输入 + `role="listbox"`。Enter / 点击提交；Escape 关闭；方向键移动。触发钮和选项 `min-height: 40px`。面板用 `position: absolute`（或打开时 `fixed` 以免被 `.workspace` 裁切）。`paint()` 不在筛选时发生，避免焦点丢失。族名进 HTML 必须 `escapeHtml`；进 `style` 必须加引号且不含已拒绝的元字符。

## Layout

空间命题：页头 → 二级导航 | 内容面。内容面是主任务区，必须吃满 `#view` 剩余高度。空底属于卡片内部，不属于 `--main`。

改动：

- `.settings-layout`：`align-items: stretch`
- `.settings-content`：`flex: 1; min-height: 0`
- `.settings-card:last-child`：`flex: 1`
- `.settings-option-row` 舒适密度 padding 从 `1.2rem 0` 收到约 `0.75rem 0`
- 字体行：单列 combobox 替换 `.font-grid`
- 主题行仍为 2×2
- `@media (max-width: 42rem)` 现有单列导航保留

五个分组共用骨架。连接分区两张卡时只拉高最后一张。不增加新卡片、不把五组拼成一页。

## Compatibility

- 已存 `system` / `yahei` / `serif` / `mono` 继续有效。
- 已存非法值 boot 时回落 `system`。
- 预览 bootstrap（非 Tauri）无列表：下拉只有系统默认 + 当前值，帮助文案说明列表在桌面运行时加载。
- 写失败错误键继续 `error.theme` + `action.check_disk`。列表失败用新键 `error.font_list`。
- 运行中新安装的字体不热刷新（范围外）。

## Trade-offs

- GDI 枚举而不是 `queryLocalFonts`：避免 WebView 权限框和空列表假成功；只覆盖 Windows，与 v1 发布范围一致。
- 族名字符串而不是继续枚举：可表达本机任意族；必须在边界做 CSS 注入校验。
- Combobox 而不是原生 `<select>`：本机族常超过一百，需要筛选；多一个控件状态，用会话缓存和「筛选不 paint」控制复杂度。

## Rollback

把 `ui_font` 改回 `system`，或删除该键。无需 migration。列表命令可停用而不影响已存值。

## Tests

- TS：`parseUiFont` 合法族名、旧别名、非法元字符、过长、`@` 前缀；`fontStack` 引号。
- Rust：`UiFont::parse` 同上；`save_ui_font` 对合法族名 round-trip；非法回落 `system`（夹具改用含 `;` 或超长串，不再用 `"nope"`）。
- Rust：`list_installed_families` 在 Windows 上非空、无 `@`、无大小写重复；含本机常见族（如 Segoe UI）则断言存在。
- i18n：zh/en 键集合仍相等。
- 布局：1200×800 与窄窗外观、连接两帧；卡片列拉高有代码规则，实拍仍标 `UNVERIFIED` 直到浏览器/WebView 复核。
