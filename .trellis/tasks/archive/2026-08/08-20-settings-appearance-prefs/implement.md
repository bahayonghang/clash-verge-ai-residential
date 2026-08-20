# 实施清单

Worktree: `.worktrees/feat-settings-appearance-prefs`  
Branch: `feat/settings-appearance-prefs`  
Merge target: current `dev` working tree

## Order

1. 扩展 `theme.rs` / `theme.ts`：三个枚举、parse、CSS apply、常量键。
2. `AppFacade` boot / bootstrap / save_*；`lib.rs` 注册三条命令；`i18n.rs` 写失败文案。
3. `BootstrapDto` TS 字段；boot 时 apply；外观分区三行控件与 click handler。
4. `styles.css`：`--ui-font` / `--ui-font-size`、`[data-density=compact]`、设置页撑满。
5. 中英 copy；parser / persist 测试。
6. 更新 `.trellis/spec/residential-monitor/frontend/view-state.md` 与 `dto-and-decoding.md`、`backend/modules-and-errors.md`。
7. `typecheck` `lint` `test` `build` 与相关 `cargo test`。
8. 提交 worktree 分支并合并回 `dev`。

## Validation

```
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib theme:: theme_tests ui_theme_persists ui_font ui_density ui_font_size
```

若过滤名不够，跑 facade 模块里新增 persist 测试 + `theme.rs` 单测。

## Risky files

- `residential-monitor/src/main.ts` 与 `styles.css` 在 `dev` 工作区已有未提交实时连接改动。worktree 从 HEAD 起步；合并时只解决外观 hunk。
- `facade.rs` 当前脏改动在 query snapshot，与新增字段分区不同，冲突面小。

## Rollback

删除三条设置键即可回到默认外观；不需要 migration。
