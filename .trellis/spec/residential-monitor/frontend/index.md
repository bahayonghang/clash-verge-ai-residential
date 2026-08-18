# residential-monitor frontend

Vanilla TypeScript + Vite 桌面壳。不引入 UI 框架。根仓库 frontend spec 只覆盖可粘贴 Clash 扩展，不适用于本子项目。

## Pre-Development Checklist

- 读 `dto-and-decoding.md`：Channel / Command 载荷在边界解码。
- 读 `view-state.md`：前端只保存视图选择和 DTO 缓存。
- 禁止 `window.__TAURI__`、eval、远程 URL 和 CDN。

## Quality Check

- `npm --prefix residential-monitor run typecheck`
- `npm --prefix residential-monitor run lint`
- `npm --prefix residential-monitor test`
- `npm --prefix residential-monitor run build`
- 关于页不得把未签名候选标成 `signed`。删除部分失败不得显示「已全部删除」。
