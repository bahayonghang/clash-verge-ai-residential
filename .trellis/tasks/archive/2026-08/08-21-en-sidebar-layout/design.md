# 设计：英文侧栏品牌与底栏排版

## 结构

只改 `sidebar.tsx`、`globals.css` 里导航相关规则、`i18n` 侧栏口号键、`sidebar.test.tsx`。宽度常量不动。

品牌区：`p-6` 改为 `p-4`，图标 `shrink-0`，标题容器 `min-w-0 flex-1`。英文 `h1` 两行：

```
<span class="block">Residential</span>
<span class="block">Traffic Monitor</span>
```

`data-brand="en-stack"` 便于测试。中文仍渲染 `product.display_name` 一行。字号用 DESIGN title（约 `1.05rem`），`leading-tight`，状态点 `shrink-0` 与第一行对齐。

口号：新键 `product.slogan_sidebar`。英文 `Observed lower bound, not a bill.` 中文可与完整口号相同或同样短句。`line-clamp-3 break-words`。关于弹层继续用 `product.slogan`。

导航与底栏：`.shell-nav-item` 增加 `whitespace-nowrap`；图标 `shrink-0`；标签包在 `span.min-w-0.truncate`。item 水平 padding 从 `px-4` 收到 `px-3`，使 220px 下 `Live connections` 与 `Settings / data` 完整单行。160px 走 truncate。

## 取舍

不改默认宽度 220，不缩短导航英文。侧栏口号短于设置页，避免把 secret 提示塞进 220px 品牌区。
