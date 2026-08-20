# 实施清单

Branch: `feat/settings-system-fonts-layout`  
Merge target: `dev`

工作区当前干净则直接在该分支改。若 `dev` 上又出现无关脏文件，再用 `git-worktree` 技能隔离到 `.worktrees/`。

实现前读 `trellis-before-dev`。改 UI 前跑 Impeccable `context.mjs --target residential-monitor/src/styles.css`，并读 `craft-floor.md` 与 `layout.md`。不要读完 `DESIGN.md` 就换视觉世界。

## Order

1. `theme.rs`：`UiFont` 改为透明字符串 newtype；`parse` / 族名校验 / `font_stack`；GDI `list_installed_families`；单测。`Cargo.toml` 为 `windows-sys` 增加 GDI feature。
2. `AppFacade::save_ui_font` 与 boot 读取改为字符串契约。更新 `ui_font_size_and_density_persist_and_fall_back`：合法族名 round-trip；非法夹具不用 `"nope"`。
3. `lib.rs` 注册 `list_ui_fonts`。`i18n.rs` 增加 `error.font_list` 与对应 action。
4. `theme.ts`：`parseUiFont` / `fontStack` / `applyFont`（`--ui-font` + `data-font=custom`）。更新 `theme.test.ts`。`dto.ts` 中 `uiFont` 改为 `string`。
5. `main.ts`：外观行换成 combobox；会话缓存列表；筛选不 `paint`；选中走现有 `save_ui_font`。`escapeHtml` 所有族名。
6. `styles.css`：工作区 stretch（`.settings-layout` / `.settings-content` / `.settings-card:last-child`）；选项行节奏；combobox；去掉 `.font-grid` 四格依赖。窄窗规则保持。
7. `zh.ts` / `en.ts`：帮助文案、列表失败、筛选占位；可保留旧四档标签供别名显示。键集合仍相等。
8. Spec：`view-state.md`、`dto-and-decoding.md`、`modules-and-errors.md` 把 `ui_font` 从四值枚举改成校验字符串。
9. 门禁命令。设置页 1200 与窄窗、外观 + 连接实操（Impeccable layout 一次批处理，桌面和窄窗一起）。

## Validation

```
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib theme::
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib ui_font
```

浏览器 / `just tdev`：外观下拉搜索、选族、系统默认、旧别名重启、列表失败回退；1200×800 与窄窗确认卡片拉高。未实拍前不得把 AC4 写成已验证。

## Risky files

- `residential-monitor/src/main.ts`：设置 HTML 字符串与 click 委托集中在此。combobox 状态必须放在 `paint` 之外。
- `residential-monitor/src/styles.css`：stretch 规则会影响五个分组；连接分区有两张卡，只拉高 `:last-child`。
- `theme.rs` + `facade.rs`：`UiFont` 不再 `Copy`，所有 `ui_font` 赋值改 clone。

## Rollback

删除或改回 `ui_font` 键即回到系统栈。`list_ui_fonts` 可移除而不影响已存值。无需 schema migration。
