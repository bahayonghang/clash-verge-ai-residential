# 填充设置关于页身份信息

## Goal

进入设置「关于」分区后立即看到仓库里真实存在的身份、签名与发布事实。用户不必先点「刷新关于」，也不再对着一张拉满的空卡。

## Dependencies and confirmed facts

- 父任务：`08-20-settings-about-sidebar`。本子任务不改应用壳宽度。
- `about` 初值 `null`（`src/main.ts:1431`）。只有 `#load-about` 才 `get_about`（`main.ts:2717-2723`）。成功后只渲染三段段落（`main.ts:945-955`）。`#open-releases` 把 URL 写入 `errorZh`（`main.ts:2732-2738`）。
- AboutDto 字段已在 `c5/about.rs` 与 `src/dto.ts`。`decodeAbout` 拒绝 `signed === true`。固定 Releases URL 在 `identity.rs`。
- 静态事实：LICENSE MIT；README / PRODUCT 写明 Windows 11 NSIS current-user、无遥测、数据只留本机。口号与日志目录已有别处入口。
- 末张设置卡 `min-height: 100%`（`styles.css:1550-1553`）。规范：关于页不得把未签名标成 signed；发布地址只展示固定 URL（`.trellis/spec/residential-monitor/frontend/view-state.md`）。

## Requirements

- 首次进入关于分区自动 `get_about` + `decodeAbout`。会话缓存；「刷新关于」强制重拉。加载中、失败、成功三态。失败保留刷新，不把空闲文案当默认。
- 用标签/值定义列表展示：产品名、版本、可执行文件、identifier、AUMID、签名状态与 `signatureNoteZh`、无应用内自动更新、无 Windows Service、固定 Releases URL（卡内可选中等宽文本）。
- i18n 只读行：MIT 许可证、Windows 11 NSIS current-user、数据只留本机且无遥测。不新增 AboutDto 字段。
- `#open-releases` 不得写 `errorZh`。URL 已在卡内。不打开浏览器，不申请 opener。
- 中英键齐全。窄窗口单列。卡片撑满工作区时内容占满空底。动态重绘恢复焦点。secret 与原始 payload 不进入关于模板。

## Acceptance Criteria

- [ ] AC1：进入关于分区后自动出现解码后的身份行；刷新可重拉；失败有下一步。
- [ ] AC2：R1 字段与三条静态事实可见；Releases URL 在卡内可选中；`errorZh` 不再被发布地址占用。
- [ ] AC3：`decodeAbout({ signed: true })` 仍抛错；界面不渲染「已签名」。
- [ ] AC4：1200×800 与窄窗口无大块空底、无水平溢出；zh/en 键集合相等。
- [ ] AC5：`npm --prefix residential-monitor` 的 typecheck、lint、test、build 通过。

## Out of scope

- 侧栏宽度、外观/连接/数据/危险分区重做。
- git hash、changelog、数据目录路径、应用内打开 GitHub。
- 扩展 AboutDto 或把 `signed` 改为 true。
