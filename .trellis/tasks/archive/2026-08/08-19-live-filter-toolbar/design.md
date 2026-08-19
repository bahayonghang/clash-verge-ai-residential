# 技术设计：实时筛选工具条

DOM 与纵向分配见父任务 `design.md` 的 Live toolbar structure。

## 本子任务落地

- 改 `renderLive` markup 与 `styles.css`。事件委托的 `id` / `data-*` 保持：`live-residential`、`live-add-clause`、`data-filter-field|mode|value|remove`。
- 关闭列按钮保持主色。添加 / 删除条件用 `.btn-secondary`。
- 不为工具条引入新 i18n 语义；可补 `aria-label` 键。

## 不改

`defaultLiveQuery`、`ConnectionFilter`、Rust 家宽针。
