# 实施计划：实时连接界面优化

## 阶段与依赖

父任务不直接启动实现；先在用户批准本规划后，按子任务顺序启动。筛选与列宽可以并行；热点摘要依赖两者确定的 applied query/重绘边界；最终集成检查在三个子任务完成后执行。

1. `08-20-live-filter-workspace`：筛选工作区与请求应用状态。
2. `08-20-live-table-width-stability`：列宽拖动状态机、固定 table width、持久化和可达性。
3. `08-20-live-connection-hotspot-summary`：Rust query/DTO + TS decoder/state + 两张卡；依赖第 1 步的 applied query 形状。
4. 父任务集成检查：跨主题/语言/空态、真实刷新重绘、summary 与 rows 同采样、并发请求和布局回归。

## 有序清单

### 0. 启动前

- [ ] 评审本 `prd.md`、`design.md`、`implement.md` 与三个子任务 PRD；仅在用户明确批准最新规划后运行对应 `task.py start`。
- [ ] 启动前重新读取 `.trellis/spec/residential-monitor/frontend/index.md`、`view-state.md`、`dto-and-decoding.md` 与本任务 research；确认无新的工作区脏改动。
- [ ] 记录当前基线：`git status --short`、`npm run check`（必要时 Rust 目标测试基线）以及现有 `live-table-layout`/`live-table-sort` 测试结果。

### 1. 筛选工作区

- [ ] 建立 applied/draft 两层状态与递增 request token；文本输入不在每次 keypress 触发整页查询。
- [ ] 重组快速筛选、已应用条件、编辑/应用/取消、清空与无匹配反馈；保留 8 条 AND、字段切换重置和数值单位换算。
- [ ] 处理应用/取消/失焦/Escape、重复点击和过期查询响应；保持 focus/selection，确保动态 paint 不泄漏条件文本。
- [ ] 补 TS 单测：draft 转 query、清空/删除、字段切换、空值忽略、token 只接受最新响应、英文/中文 label key。

### 2. 固定列宽

- [ ] 将 resize handle 变成可捕获 pointer 的稳定交互，统一 pointerup/cancel/lostpointercapture/blur 收尾并只保存一次。
- [ ] 确认 wrapper/table/colgroup CSS：显式 table pixel width + `table-layout: fixed` + 横向滚动；刷新、排序、隐藏列和主题切换不重新按内容测量。
- [ ] 保持 clamp、至少一列可见、非法布局默认回退和 `live_table_layout` 持久化；补键盘/焦点说明或等价可达操作。
- [ ] 补 TS 单测与手动检查：拖动目标列、取消拖动、连续拖动、保存失败、重绘后滚动位置与列宽一致。

### 3. 热点摘要

- [ ] 在 `c2/query.rs` 保持原 filter/排序/分页语义的同时，从完整 matched 集合计算 matchedCount、sampleUtc、topDownload、topUpload，并写稳定 tie-break 测试。
- [ ] 在 Rust facade/command 与 `src/ipc/live-session.ts` / `src/dto.ts` 扩展 ConnectionPage 契约；decoder 对缺字段、null、未知能力 fail closed，不将未知写成 0。
- [ ] 将 summary 与 rows 在同一 hub snapshot 中返回；前端 `refreshLivePage` 一次接收并在动态重绘时保持同一 sample 状态。
- [ ] 渲染两张方向热点卡：主机/进程/目标标签、方向值、样本时间、状态；无匹配/暂停/缺口/未连接有专门文案；图标若有则同时提供文本。
- [ ] 补 Rust query tests、TS decoder/render tests、secret/原始 payload 防泄漏检查。

### 4. 集成与质量门

- [ ] 检查筛选应用会重置 cursor 且 summary 仍代表同一 filter；排序仅影响 rows 顺序，不改变两个热点口径。
- [ ] 检查 monitor bootstrap/connectionDelta/resync、close marks、collector pause 和空态未回归。
- [ ] 检查中英、latte/frappe/macchiato/mocha、1200×800 与窄窗口、focus-visible、prefers-contrast、prefers-reduced-motion。
- [ ] 运行 `npm run check`、Rust 目标测试/`cargo fmt --check`（若修改 Rust）、`git diff --check`；必要时运行仓库约定的 `just ci`。
- [ ] 仅报告自动化结果；Windows 安装态、真实 Clash Verge Rev/Mihomo、拖动实拍和屏幕阅读器结果分别标注 PASS/UNVERIFIED。

## 验证命令

```powershell
cd residential-monitor
npm run typecheck
npm run lint
npm test
npm run build
npm run check
cd ..
git diff --check
```

如触及 Rust query/DTO：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml c2::query
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml -- --check
```

## 风险与回滚点

- `ConnectionPage` 扩展是跨 Rust/TS 契约变更：先加纯 query 测试与 decoder，再接 UI；任何不一致优先回滚摘要字段/卡片，不改变已验证的筛选/列宽。
- `paint()` 与输入焦点耦合：若应用/取消模型导致焦点回归，退回局部 DOM 更新或保留 draft 并延后整页 paint，禁止恢复“每次按键即 query”。
- pointer capture 在 Tauri WebView 上的差异：保留现有 window pointermove 作为安全 fallback，但不得双重提交；取消事件必须清除 dragging。
- 旧本机 `live_table_layout` payload：继续经 sanitize，不能在新版本中把非法宽度传播到 table style。
