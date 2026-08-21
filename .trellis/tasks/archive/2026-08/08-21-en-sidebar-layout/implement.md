# 实施：英文侧栏品牌与底栏排版

## 顺序

1. `en.ts` / `zh.ts` 增加 `product.slogan_sidebar`；`i18n/index.test.ts` 键集合断言会跟上。
2. `sidebar.tsx`：品牌两行、口号键、nav/footer `shrink-0` + `truncate`、padding。
3. `globals.css`：若 `.shell-nav-item` 需要 `whitespace-nowrap`，写在现有规则里。
4. `sidebar.test.tsx`：英文 220 含两行品牌与 `Live connections` / `Settings / data`；中文含「家宽流量监控」且无 `data-brand="en-stack"`。
5. `npm --prefix residential-monitor run check`。

## 风险文件

- `sidebar.tsx` 关于弹层误用短口号。
- 密度 compact 下导航高度仍由 `--nav-item-py` 控制，不要另写死像素高度把点击目标压没。

## 开工前

- 不改 `SHELL_WIDTH_*`。
- 不改未知主机逻辑。
