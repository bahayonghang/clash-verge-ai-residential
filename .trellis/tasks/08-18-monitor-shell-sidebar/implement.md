# 家宽监控应用壳侧栏与界面重构 — 实施

## Checklist

1. 用户批准本规划摘要后执行 `python ./.trellis/scripts/task.py start 08-18-monitor-shell-sidebar`。未批准不改产品代码。
2. 加载 `trellis-before-dev` 与 Impeccable new-work（Operate，替换视觉世界）。先选定方向，再写壳层。
3. 若 `.impeccable/config.json` 仍无 `buildPath`，单独问一次：comp-first 或 code-first。用户的回答写入该文件。未回答则本轮走 comp-first，且不写入配置。
4. 按选定世界生成 6 张本地图标，写入 `residential-monitor/src/assets/`。核对 CSP 与深色/高对比可辨认。
5. 改 `renderApp` / `navHtml`：左侧栏 + 主区。口号移到侧栏。Recovery-only 去掉五页假入口。
6. 改各 `render*`：删除重复页面标题，保留区块标题和全部既有控件 `id`。
7. 重写 `styles.css`：新 token、侧栏、指标、表格、表单、状态、打印、高对比。删除旧 `--bg` / `--panel` / `--accent` 依赖。
8. 补导航测试：五条 route 仍稳定；侧栏项同时暴露标题；禁用态不可用。
9. 跑质量门。用桌面 1200 宽和一次更窄宽度检查首屏密度（AC7）与键盘焦点。

## Validation

```text
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
```

必要时再跑 `just monitor-check`。根 `just ci` 仅在改动可能影响根扫描时运行。

## Risky files

- `residential-monitor/src/main.ts`：整页 `innerHTML` 重绘。改壳时必须保留全部既有 `id` 和 `data-close` / `data-route`。
- `residential-monitor/src/styles.css`：全文件视觉替换。勿丢掉 skip、focus、print、contrast。
- 不要改 `dto.ts`、`ipc/decoder.ts`、`ipc/reducer.ts`、Rust 源，除非发现壳层无法避免的契约缺口。发现缺口先回规划，不在实施中扩 scope。

## Rollback points

- 方向选择未锁定：停在无产品代码状态。
- 图标或 CSS 破坏 CSP / 构建：删新增资源，恢复可构建的上一版样式。
- 质量门失败：先修测试与选择器，不改 DTO 让测试变绿。
