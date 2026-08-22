# 执行计划：修复设置页字体选择器重绘打断

Branch: `feat/settings-system-fonts-layout`
Merge target: `dev`

继续在当前分支改。不要从 `main` 另开分支。`main` 上没有字体下拉。

## 顺序

四步按缺陷推进。步骤 1 单独不能修采集运行时的列表滚动，必须与步骤 2 一起交付。

### 步骤 1：字体列表成为滚动容器（R1 / AC1）

- `residential-monitor/src/main.ts:929` — `#font-picker-filter` 补 `class="font-picker-filter"`。
- `residential-monitor/src/main.ts:930` — `#font-picker-list` 补 `class="font-picker-list"` 与 `data-report-scroll="font-picker-list"`。
- 默认不改 `styles.css`。`sm` 字号下面板仍溢出时，给 `.font-picker-panel` 加 `overflow: hidden`，给 `.font-picker-list` 加 `min-height: 0`。

验证：`npm --prefix residential-monitor run build`。手工确认下拉不再撑高 workspace、列表内部可滚。

### 步骤 2：设置页跳过无关推送重绘（R2 / R5 / AC2 / AC4 / AC6）

- `residential-monitor/src/format/report-inspect.ts`
  - 保留 `SKIP_REPORT_PAINT_KINDS` 原集合。
  - 新增 `SKIP_SETTINGS_PAINT_KINDS`：`connectionDelta` / `summaryChanged` / `alertChanged`。
  - `shouldSkipReportPaint` 改名 `shouldSkipRoutinePaint`，判定顺序：错误文案变化 → `false`；`reports` → 查 report 集合；`settings-data` → 查 settings 集合；其余 → `false`。
- `residential-monitor/src/main.ts:100` — import 跟随改名（该 import 块按字母序，`shouldSkipRoutinePaint` 排在 `reportInspectModel` 之后、`trendInspectKey` 之前，位置不变）。
- `residential-monitor/src/main.ts:1704` — 调用点改名，位置不动。
- `residential-monitor/src/main.ts` 分区切换 — `nextSection === "connection"` 且当前不是连接分区时，`await refreshLivePage()` 再 `paint()`。
- `residential-monitor/src/format/report-inspect.test.ts` — 跟随改名，补用例：
  - `settings-data` + `connectionDelta` / `summaryChanged` / `alertChanged` → `true`。
  - `settings-data` + `healthChanged` → `false`。
  - `settings-data` + `bootstrap` → `false`。
  - `settings-data` + `connectionDelta` + 错误文案变化 → `false`。
  - `live` / `overview` + `connectionDelta` → `false`。
  - 既有 reports 用例断言不变。

验证：`npm --prefix residential-monitor test`。

### 步骤 3：焦点选区与滚动写回（R3 / R4 / AC2 / AC3）

- `residential-monitor/src/main.ts:1294` — `focusedId` 扩为局部结构，`document.activeElement` 是 `HTMLInputElement` / `HTMLTextAreaElement` 时一并记 `selectionStart` / `selectionEnd` / `selectionDirection`。
- `residential-monitor/src/main.ts:1348-1350` — `focus({ preventScroll: true })`；`selectionStart` 非 `null` 时在 `try/catch` 内 `setSelectionRange`。
- `residential-monitor/src/main.ts:1360` — 条件放宽为 `route === "reports" || route === "settings-data"`；`reportInspectPinned` / `reportInspectHover` 清理留在 `reports` 分支内。字体列表滚动由步骤 1 的 `data-report-scroll` 自动进入 `wraps`。

验证：`npm --prefix residential-monitor run typecheck`。

## 验证命令

```bash
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
just ci
```

## 手工验收（AC1 / AC2 / AC3 / AC4）

需要采集器运行、有实时连接流量。

1. 进设置页 → 外观与语言 → 等到字体列表加载完成 → 打开字体下拉。确认面板自身限高、不再撑高 workspace，列表在内部滚动，能滚到末尾字体并点选。字号切到 `sm` 再确认一次。
2. 在设置页停留 ≥ 30 秒不操作，滚动字体列表和 workspace，确认位置不被重置。健康状态变化触发重绘后，列表滚动写回。
3. 搜索框输入 `lxgw`，把插入符移到中间再输入一个字符，确认插入符停在插入位置。用中文输入法输入一次，确认组合不被整页重建打断。列表加载完成后再输入，插入符仍停在插入位置。
4. 断开或恢复控制器连接，确认连接分区的连接状态徽标随健康状态变化更新。从外观切到连接分区，确认采集器文案为进入时的当前值。
5. 从设置页切到实时页，确认连接列表是当前数据。
6. 报告页、告警页操作一遍，确认行为不变。

## 回滚点

- 步骤 1 后：只回退 markup class 与 `data-report-scroll`。
- 步骤 2 后：改名、跳过集合与连接分区补拉可整体 revert，不牵动步骤 1、3。
- 步骤 3 后：选区与滚动写回可整体 revert。

## 审查门

- 步骤 2 完成、单测通过后停一次，向用户报告跳过集合的最终取值，再进步骤 3。
- 全部步骤完成、`just ci` 通过后停一次，等用户完成手工验收，再进 Phase 3 提交。
