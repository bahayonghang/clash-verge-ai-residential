# 实施计划：设置关于页与侧栏宽度

## 阶段与依赖

父任务不直接启动实现。用户批准本规划后，按顺序启动子任务。

1. `08-20-settings-about-identity`：自动加载、定义列表、静态事实、发布地址离开 `errorZh`。
2. `08-20-shell-sidebar-resize`：`--shell-width`、拖动/键盘、`ui_sidebar_width`、Bootstrap 恢复。
3. 父任务集成检查：关于卡与侧栏共存于 1200×800、四主题、中英、拖动与设置 skip-paint、`signed` 断言。

## 有序清单

### 0. 启动前

- [ ] 用户明确批准最新规划摘要。
- [ ] 只 `task.py start` 当前子任务，不 start 父任务。
- [ ] 读 `.trellis/spec/residential-monitor/frontend/index.md`、`view-state.md`、`dto-and-decoding.md`、`backend/modules-and-errors.md` 与 `research/about-and-sidebar-evidence.md`。
- [ ] 记录基线：`git status --short`。工作区已有无关的 `residential-monitor/src-tauri/Cargo.toml` 改动，实施时不要纳入本任务提交。

### 1. 关于页

见子任务 `08-20-settings-about-identity/implement.md`。

### 2. 侧栏宽度

见子任务 `08-20-shell-sidebar-resize/implement.md`。须在关于页改动合入后再改同一批前端文件。

### 3. 集成

- [ ] 关于自动加载与侧栏拖动同时可用；拖动中 `paint` 被挡住时，松开后身份行仍在。
- [ ] 设置页 `connectionDelta` skip-paint 仍成立；进入关于仍能加载。
- [ ] 1200×800 与窄窗口、四主题、中英、focus-visible、prefers-contrast、reduced-motion。
- [ ] `decodeAbout` 拒绝 `signed: true`；删除部分失败断言仍在。
- [ ] `npm --prefix residential-monitor run typecheck && lint && test && build`；若改 Rust，再跑对应 `cargo test` 与 `fmt --check`。
- [ ] 真实 Tauri WebView 拖动与关于加载标为 PASS 或 UNVERIFIED，不拿 Vite 预览冒充安装态。

## 验证命令

```powershell
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
git diff --check
```

侧栏持久化触及 Rust 时：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml -- --check
```

## 风险与回滚点

- `main.ts` 同时承担设置渲染与壳 markup：子任务必须小 diff，避免互相覆盖未提交改动。
- WebView pointer capture 与实时列宽相同：取消路径必须清 dragging。
- 旧库无 `ui_sidebar_width`：缺字段即 220，行为与现在视觉接近。
- 无关 `Cargo.toml` 脏改动：提交时只 stage 本任务文件。
