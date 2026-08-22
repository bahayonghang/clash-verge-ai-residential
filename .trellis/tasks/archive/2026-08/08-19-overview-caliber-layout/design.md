# 技术设计：概览成对口径

结构与网格规则见父任务 `design.md` 的 Overview structure。

## 本子任务落地

- 只改 `renderOverview` 与相关 CSS / i18n。
- 新增键：组名、`overview.other_down`、`overview.gap_down`、`overview.over_down`、分类列表头。
- 分类行纯函数可单测：并集键、缺侧「未知」、空 map → 空列表。
- `.overview` 使用主题语义 token，不写死旧灰卡色。

## 不改

`decodeOverview` 字段、Channel snapshot 形状、核算。
