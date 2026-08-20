# 设计：应用壳侧栏可调宽度

## Boundaries

- 新键 `ui_sidebar_width` 与 `ui_theme` 同层：`put_setting`，不进控制器 JSON，不升 schema。
- 新命令 `save_ui_sidebar_width(width: number | string) -> number`，返回 clamp 后的整数。
- 前端模块可新增 `src/shell-width.ts`（parse / clamp / 默认），模式对齐 `live-table-layout.ts` 的 sanitize，避免把数字逻辑散落在 `main.ts`。

## Contract

| 项 | 值 |
|---|---|
| 默认 | 220 |
| 最小 | 160 |
| 最大 | 352 |
| 步进 | 8 |
| 单位 | CSS px |

Rust 与 TS 使用同一数字范围。非有限、非整数先 round 再 clamp。字符串只接受十进制整数。

`BootstrapDto` 增加 `uiSidebarWidth?: number`。前端 `parseUiSidebarWidth`：缺字段 → 220。

Recovery 无 storage：`save_ui_sidebar_width` 更新内存字段并返回规范化值，不调用 `put_setting`。

## Interaction

```text
pointerdown on handle → setPointerCapture, record startX/startWidth
pointermove → width = clamp(startWidth + dx) → html.style --shell-width, 更新 aria-valuenow
pointerup → 若 changed 则 invoke save；清 dragging
pointercancel / lostpointercapture / blur → 恢复 startWidth，不保存
```

`paint()` 在 `shellDragging || liveTableDragging` 时 return。两个标志不得同时为真。

键盘：handle `tabindex="0"`。ArrowLeft 减小、ArrowRight 增大、Home 最小、End 最大。keydown 改内存与 CSS；keyup 或 blur 时若有变化则保存。

CSS：

```css
.shell {
  width: var(--shell-width, 220px);
  flex: 0 0 var(--shell-width, 220px);
}
```

手柄绝对定位在壳右缘，宽度约 6–8px，命中区至少 40px 高（可拉长整列）。`cursor: ew-resize`。不要用 `transition: all`。

## Tests

- TS：parse 缺字段、`"220"`、`12.9`、`NaN`、`159`、`353`。
- Rust：persist round-trip；非法回落 220；无 storage 只改内存。
- 回归：实时列宽测试不受新标志影响。

## Compatibility

无该键的旧库启动即为 220px，与当前 13.75rem@16px 相同。字号 sm/lg 不再改变侧栏像素宽度。
