# 设计：外观字体 / 字号 / 密度

## Boundaries

- Rust 仍是设置键权威：解析、回落、`put_setting`。前端只把合法枚举写到 `document.documentElement`。
- 不进控制器 JSON。不新增 SQLite 表或 schema version。
- 新 Tauri 命令与 `save_ui_theme` 同形：`save_ui_font` / `save_ui_font_size` / `save_ui_density`。

## Contracts

| 键 | 合法值 | 默认 |
|---|---|---|
| `ui_font` | `system` `yahei` `serif` `mono` | `system` |
| `ui_font_size` | `sm` `md` `lg` | `md` |
| `ui_density` | `comfortable` `compact` | `comfortable` |

`BootstrapDto` 增加 camelCase 可选字段 `uiFont` `uiFontSize` `uiDensity`。前端 `parse*` 缺字段按默认。

应用面：

- `html[data-font]` + `--ui-font`
- `html[data-font-size]` + `--ui-font-size`
- `html[data-density]`，compact 用更紧的 padding/gap 规则

字体栈（本机，无 CDN）：

- `system`: `"Segoe UI", "Microsoft YaHei", sans-serif`
- `yahei`: `"Microsoft YaHei UI", "Microsoft YaHei", sans-serif`
- `serif`: `"Source Han Serif SC", "Noto Serif CJK SC", "Songti SC", SimSun, serif`
- `mono`: `"Cascadia Mono", "Sarasa Mono SC", ui-monospace, monospace`

字号：`sm=14px` `md=16px` `lg=18px`。`html, body { font-family: var(--ui-font); font-size: var(--ui-font-size); }`。按钮 `min-height` 用 `40px`，避免 rem 缩放破坏命中区。

## Data flow

```
设置点击 → invoke save_ui_* → AppFacade 解析并 put_setting → 返回规范化字符串
        ↘ 失败时前端仍 apply*（与主题相同）
boot → BootstrapDto 三字段 → parse* → apply*
```

Recovery 无 `storage` 时 `save_*` 只改内存字段，下次普通启动不会带回。

## Compatibility

- 旧库没有这三键：boot 走默认，行为与现在一致。
- 预览 bootstrap（非 Tauri）带默认值。
- 写失败错误键复用 `error.theme` 或新增 `error.appearance`；下一步仍是 `action.check_disk`。

## Layout

- `#view:has(.settings-page)` 纵向 flex；`.settings-page` / `.settings-layout` 撑满工作区。
- 外观卡片靠新增三行填满上半屏空白，不堆装饰。

## Tests

- TS：`parseUiFont` / `parseUiFontSize` / `parseUiDensity` 合法与回落。
- Rust：三键 persist + 非法回落，模式对齐 `ui_theme_persists_and_falls_back_to_mocha`。
- i18n：zh/en 键集合仍相等。
