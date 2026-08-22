# 实施计划：设置关于页身份信息

## 有序清单

- [ ] 规划已批准；`task.py start 08-20-settings-about-identity`。读父任务 `prd.md` / `design.md` 与 `research/about-and-sidebar-evidence.md`。
- [ ] 增加关于加载会话状态（in-flight / error）。在进入 about 时调用现有 `get_about`，解码后写入 `about`。
- [ ] `renderSettings` 把三段 `<p>` 换成定义列表；补 i18n 静态行与 loading/fail 键。
- [ ] `#open-releases` 停止写 `errorZh`；选中卡内 URL。
- [ ] CSS：`.about-list` 两列/窄屏一列；末卡仍 `min-height: 100%`。
- [ ] 测试：`decodeAbout` signed 断言、zh/en 键。手动：进入关于无需点击即可看到版本与 URL。
- [ ] `npm --prefix residential-monitor run typecheck && lint && test && build`。

## 验证命令

```powershell
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
```

## 回滚

还原 `renderSettings` 关于块与 `#load-about` / `#open-releases` 处理器即可。不涉及 Rust schema。
