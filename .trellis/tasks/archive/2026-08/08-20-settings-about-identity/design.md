# 设计：设置关于页身份信息

## Boundaries

- 命令仍是 `get_about` / `open_releases`。AboutDto schemaVersion 保持 1，不新增字段。
- 静态许可证/平台/隐私只走 i18n。Rust `signature_note_zh` 保持中文。
- 不改 `.shell` 宽度、不改 skip-paint 规则的判定集合，只增加进入关于时的一次加载。

## Data flow

```text
settingsSection → about
    aboutCache empty and not in-flight → invoke get_about
        ok → decodeAbout → about = dto → paint
        throw → aboutError 文案 → paint（about 仍为 null）
刷新 → 清空缓存再走同一路径
```

加载中在卡片内显示 `settings.about_loading`。失败显示 `settings.about_fail` 与刷新。禁止回到默认 idle 作为主路径。

进入关于的触发点与外观拉字体对齐：`settingsSection` 切到 `about` 时、以及 `route` 进入 `settings-data` 且当前分区已是 `about` 时。`paint()` 内不要无条件 invoke。

## Layout

`.about-body` 改为 `<dl class="about-list">`。宽屏 `grid-template-columns: minmax(8rem, 12rem) minmax(0, 1fr)`，窄窗口（现有 settings 断点）一列。等宽值用 `.mono-value`。签名说明可跨两列。

`#open-releases`：查询 `.about-release-url` 并 `selectNode` / 设置选区；若尚未加载成功则先触发加载。任何路径都不调用 `apply({ errorZh: ... })` 来展示 URL。

## Tests

- 保留 `decodeAbout` 拒绝 `signed: true`、接受 `signed: false`。
- i18n zh/en 键相等，覆盖新 about 键。
- 纯渲染单测可选：有 dto / 加载中 / 失败三种 markup 都不含 idle 主文案、不含 `errorZh` 发布句。

## Compatibility

旧行为「必须点刷新」消失。刷新按钮保留给重读。`open_releases` 命令可继续存在，前端可以不再依赖其返回值填错误条。
