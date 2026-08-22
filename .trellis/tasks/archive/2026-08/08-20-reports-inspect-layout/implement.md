# 实现清单：分析报告探查与排版

## Ordered checklist

1. **无关键重绘**  
   改 `handleMonitorRaw`：`route === "reports"` 时 `connectionDelta` / `healthChanged` / `summaryChanged` / `alertChanged` 不 `refreshLivePage`、不 `paint`，除非 `errorZh` 变化。补回归：用抽出的谓词测「reports + connectionDelta → skip」。

2. **会话 UI 状态**  
   捕获/写回：`workspace` 滚动、`data-report-scroll` 三个容器、`.report-notes` 的 `open`、钉住 key。`renderApp` 在现有 live-table 恢复旁调用。给趋势/Top N/档案 wrap 加 `data-report-scroll`。离开 `reports` 清空；换 `reportSnapshotToken` 时表滚动归零、无效钉住清除。

3. **SVG 命中区**  
   扩展 `PieSlice` 与趋势生成：写入 `data-inspect` 和短 `aria-label`。单桶柱可聚焦。多桶提供可命中的桶映射，不靠 2px polyline。纯函数测试：标签进 SVG、零值仍不画、其余扇区 key=`remainder`。

4. **探查交互**  
   在 `#app` 上委托 `pointerover` / `click` / `keydown`。临时探查与钉住共用同一套 class。`#report-inspect-tip` 用 `position: fixed`。表行与图 key 对齐。Escape 取消钉住。不改 `ReportQuery`。

5. **排版**  
   `.workspace:has(.reports)` 与 live-page 一样锁外壳滚动。查询 / 结果 / 档案三块。指标条并入结果顶。Top N 扇形作色例 + 伸展表体。容器 `48rem` 改一列。四口味 token，不新增色。

6. **i18n**  
   中英成对：探查句、钉住状态。空 label 继续走「未知」。

7. **门禁**  
   `npm --prefix residential-monitor run typecheck`、`lint`、`test`、`build`。`git diff --check`。

## Validation commands

```bash
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
```

实机：采集运行中打开分析报告，滚 Top N、展开 details、悬停并钉住扇区，等待连接增量，确认三态仍在。1200×800 与窄主区各看一次。未跑实机的项标 `UNVERIFIED`。

## Risky files / rollback points

| 文件 | 风险 |
|---|---|
| `residential-monitor/src/main.ts` | `paint` / Channel 是实时页生命线。跳过条件写错会让实时表停更，或报告页继续被冲掉。 |
| `residential-monitor/src/styles.css` | `:has(.reports)` 锁滚动若未给结果区 `min-height:0`，表体会无法内滚。 |
| `residential-monitor/src/format/report-svg.ts` | 破坏现有单测（整圆、两弧、单桶柱、空 series）。 |

回滚顺序见 `design.md` Rollback。可先只交 1–2（稳定），再交 3–5（探查+排版）。

## Follow-up before `task.py start`

- Q1 已确认：点击钉住，不下钻。
- jsonl 已写入 spec + 本目录 research。
- 不改 `DESIGN.md` 漂移。
- 实现前读 `trellis-before-dev` 与 frontend checklist。
