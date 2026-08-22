# 实施计划：重构设置与数据管理界面

## 前置门禁

- [x] 父任务规划获用户独立批准后再 `task.py start`；本子任务不自行启动。
- [x] 运行 `trellis-before-dev` 并读取 frontend / cross-layer spec；UI 写入前读取 `C:\Users\lyh\.agents\skills\impeccable\reference\craft-floor.md`。
- [x] 重新核对当前 `main.ts` / `styles.css` 以及并行托盘任务状态，保留 unrelated dirty paths。

## 步骤

1. [x] 设计并测试 `SettingsSection` / `SettingsDraft`，确认 section 切换、draft 保留、secret 不进入 HTML。
2. [x] 将 `renderSettings` 拆为 settings layout + secondary nav + 五个纯 section renderer；默认连接 section。
3. [x] 接入权威 health / collector 状态与现有 save/test/reconnect/disconnect command；明确单帧测试和持续监控差异。
4. [x] 将四主题改为主题选项卡片，将语言改为分段控件；更新中英 i18n，保留现有 theme persistence。
5. [x] 重新编排数据 / 关于 / 危险区，保持 backup / restore / retention / vacuum / delete 的现有语义和 Recovery 分支。
6. [x] 编写 settings-scoped CSS：桌面两栏、窄窗导航、表单网格、状态行、危险区、focus / disabled / loading / error / success、reduced motion。
7. [x] 做一次 Impeccable detector；修复机械问题后，不重复 detector；准备 finish reviewer 输入包。

## 验证

- [x] `npm --prefix residential-monitor run typecheck`
- [x] `npm --prefix residential-monitor run lint`
- [x] `npm --prefix residential-monitor test`
- [x] `npm --prefix residential-monitor run build`
- [x] `node C:\Users\lyh\.skillsmanage\skills\impeccable\scripts\detect.mjs --json residential-monitor/src/main.ts residential-monitor/src/styles.css`
- [ ] 通过 `just tdev` / Tauri WebView 采集 1200×800 与窄窗口截图；检查截图不是 fixture、不是黑屏、不是半加载（当前已有安装实例占用 single-instance，未能取得当前构建窗口）。
- [ ] 键盘走查 section nav、地址 / secret、保存、重连、危险删除；走查 Mocha / Latte 与中 / 英、loading / error / reduced-motion（依赖真实窗口）。
- [ ] 独立 finish reviewer 复核截图和 craftsmanship；截图缺失，已用独立 in-thread Trellis check + detector 降级复核。

## 回滚点

- 先回滚 settings view module / markup / CSS / i18n，保留自动连接 child 的 Rust 改动。
- 不改数据库 schema、Credential Manager、command contract 或其他四个业务页。
