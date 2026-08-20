# 技术设计：修复设置页字体选择器重绘打断

## 改动清单

| 文件                                                    | 改动                                                                              |
| ------------------------------------------------------- | --------------------------------------------------------------------------------- |
| `residential-monitor/src/main.ts`                       | 补两个 class 与 `data-report-scroll`；扩展重绘跳过调用；焦点选区与滚动写回；进入连接分区补拉 tray |
| `residential-monitor/src/format/report-inspect.ts`      | `shouldSkipReportPaint` 泛化并改名为 `shouldSkipRoutinePaint`，新增设置页跳过集合 |
| `residential-monitor/src/format/report-inspect.test.ts` | 跟随改名，补设置页用例                                                            |

默认不改 `styles.css`：`.font-picker-filter`（1808）与 `.font-picker-list`（1813）的规则本身正确，缺的是 markup 上的 class。`sm` 字号下面板仍溢出时，再给 `.font-picker-panel` 加 `overflow: hidden`、给 `.font-picker-list` 加 `min-height: 0`。

实现分支：`feat/settings-system-fonts-layout`。合并目标：`dev`。

## 缺陷 1：列表不是滚动容器

`main.ts:929-930` 两处元素补 class，列表再加滚动标记，与同段 markup 里 `font-picker` / `font-picker-trigger` / `font-picker-panel` 的写法对齐：

```
<input class="font-picker-filter" id="font-picker-filter" type="search" ...>
<div class="font-picker-list" role="listbox" id="font-picker-list" data-report-scroll="font-picker-list" ...>
```

补上后列表拿到 `max-height: 14rem; overflow: auto`，成为独立滚动容器。面板是 `position: absolute`，相对 `.font-picker`。卡片与 `.settings-content` 不裁切溢出。限高约束的是面板自身，不要求面板底边落在设置卡片可视区内。

高度核算依赖 `html` 的 `--ui-font-size`。内边距与列表限高用 `rem`，搜索框最小高度用 `40px`。`md`（16px）时余量约 8px；`sm`（14px）时余量约 2px。先只补 class；`sm` 仍溢出再改 CSS。

列表未到边界时滚轮滚动列表。滚到边界后滚轮可以传给 `.workspace`。不加 `overscroll-behavior: contain`。

## 缺陷 2：设置页每秒整页重绘

### 契约

`format/report-inspect.ts` 中现有函数泛化：

```ts
export function shouldSkipRoutinePaint(
  route: string,
  messageKind: string,
  errorZh: string | null,
  paintedErrorZh: string | null,
): boolean;
```

判定顺序：

1. `errorZh !== paintedErrorZh` → `false`。错误文案变化必须重绘，优先级高于路由规则。
2. `route === "reports"` → 返回 `SKIP_REPORT_PAINT_KINDS.has(messageKind)`，集合保持 `connectionDelta` / `healthChanged` / `summaryChanged` / `alertChanged` 不变。
3. `route === "settings-data"` → 返回 `SKIP_SETTINGS_PAINT_KINDS.has(messageKind)`，集合为 `connectionDelta` / `summaryChanged` / `alertChanged`。
4. 其余路由 → `false`。

两个集合分开定义。设置页集合不含 `healthChanged`，因为连接分区徽标取 `overview.health.session`。外观分区不读 `overview`，该 kind 在外观分区仍会整页重绘，这是既定抑制范围的代价。`bootstrap` 不在任何集合里，仍重绘。`summaryChanged` 当前没有 Rust 构造点，放入集合与报告页对齐。

改名理由：函数在本轮之后同时管 reports 与 settings-data 两条路由，名字里的 `Report` 会误导读者。改名只牵动已经要改的 `main.ts` 与同文件测试，改动面不扩大。文件名仍为 `report-inspect.ts`。

### 调用点

`main.ts:1704` 改为调用新名。调用位置不动，仍在 `refreshLivePage()`（1707-1709）之前，所以跳过重绘时一并跳过该次实时页拉取。

### 采集器文案

`collectorRunning` 只在 `refreshLivePage()` 里更新。跳过 `connectionDelta` 后，停留设置页期间该值冻结。

进入连接分区时（`data-settings-section="connection"` 且即将离开其他分区）先 `await refreshLivePage()` 再 `paint()`，进入当下的采集器文案为当前 tray 值。不在 skip 路径上做徽标增量更新。切到 `live` 仍走既有 `refreshLivePage()`。

### 数据不丢

`reduceMonitor`（`main.ts:1693`）在 skip 判断之前执行，`state` 照常更新。跳过的只是渲染，下一次 `paint()` 用最新 `state`。切到 `live` 路由时 `main.ts:2359-2361` 重新 `refreshLivePage()`，与 reports 路由现有行为一致。

输入法合成：跳过高频 `paint()` 后，合成过程不再被 `innerHTML` 拆掉。`setSelectionRange` 不恢复合成。

## 缺陷 3：焦点选区与滚动写回

### 捕获（`main.ts:1294` 附近）

把现有的 `focusedId` 单值扩成一个局部结构：记录 `id`，并在 `document.activeElement` 是 `HTMLInputElement` 或 `HTMLTextAreaElement` 时一并记 `selectionStart` / `selectionEnd` / `selectionDirection`。对不支持选区的 input 类型（`number`、`range` 等），`selectionStart` 读出 `null`，捕获阶段不会抛错。

### 恢复（`main.ts:1348-1350` 附近）

```
focus({ preventScroll: true })
```

再在 `try/catch` 内 `setSelectionRange(start, end, direction)`。`setSelectionRange` 对不支持选区的 input 类型会抛 `InvalidStateError`，用 `try/catch` 兜住即可，不做类型白名单——白名单要枚举 input 类型，比 `try/catch` 长且要跟随新类型维护。捕获到的 `selectionStart` 为 `null` 时直接不调用。

该恢复对全部路由生效，调用点仍在 `renderApp`。`preventScroll` 去掉 focus 自带的 scroll-into-view：没有它，focus 会先把输入框拉进视口，随后的滚动写回又要把 workspace 拉回去，产生一次可见跳动。

`loadAppearanceFonts` 完成后的 `paint()` 走同一条恢复路径。外观首次打开时操作员应等到列表加载完成再输入；加载完成那次重绘仍写回选区与列表滚动。

### workspace 与列表滚动

`main.ts:1360` 的条件由 `route === "reports"` 放宽到 `route === "reports" || route === "settings-data"`。

- 采集侧不动：`readReportScroll`（`report-inspect.ts:119`）已经无条件读 `.workspace.scrollTop` 与 `[data-report-scroll]`。
- 字体列表带 `data-report-scroll="font-picker-list"` 后，`wraps` 含该项；面板关闭时节点不在 DOM，`wraps` 写回为空操作。
- `reportInspectPinned` / `reportInspectHover` 的清理留在 `route === "reports"` 分支内，不随之放宽。
- 函数名不改。它们已经是通用 DOM 滚动读写，但与 `reportScrollReset`、`applyReportScrollReset` 同属一套命名，单独改名会牵动更多无关调用点。

执行顺序：focus 恢复（1348-1350，带 `preventScroll`）在滚动写回（1355-1361）之前，最终位置由滚动写回决定。

## 兼容性

- reports 路由的跳过集合、inspect 清理、滚动行为不变。
- live / alerts / overview 路由不受影响：新函数对这些路由返回 `false`。
- 不改 DTO、解码器、reducer、Rust 侧、`ui_font` 取值与保存路径。
- 不新增依赖。

## 验证边界

vitest 是 node 环境（`vite.config.ts` 未配 jsdom），`format/*.test.ts` 只覆盖纯函数。因此：

- 自动化覆盖：`shouldSkipRoutinePaint`。
- 手工覆盖：列表内部滚动（含 `sm`）、滚动位置保持、插入符位置、中文输入法组合、字体列表加载完成后再输入、切到连接分区的采集器文案、切回实时页数据新鲜度。

## 回滚

四处改动可单独回退：

1. 两个 class 与 `data-report-scroll` —— 回退后列表恢复溢出。
2. 跳过集合 —— 回退后设置页恢复每秒重绘。
3. 进入连接分区补拉 tray —— 回退后采集器文案只随 `connectionDelta` 更新。
4. 选区与滚动写回 —— 回退后恢复只按 id focus，且设置页不写回滚动。

## 已考虑不做

- `overscroll-behavior: contain`：列表滚到边界后滚轮可以传给 `.workspace`。重绘不再把滚动打回顶部，传递不造成跳回。
- 把整页 `innerHTML` 重建改成增量更新：范围外，风险远大于本轮收益。
- 把焦点恢复扩展到字体列表选项按钮：选项没有 id，且选中后面板即关闭（`main.ts:2283`），不需要。
- 只在 `fontPickerOpen` 时抑制重绘：已由用户否决，其余设置输入框会留下同样的打断。
- 外观分区打开时跳过 `healthChanged`：抑制范围已定为整条 `settings-data` 路由的固定集合。
- 停留设置页期间持续刷新 `collectorRunning`：不在 skip 路径上做增量 DOM。
