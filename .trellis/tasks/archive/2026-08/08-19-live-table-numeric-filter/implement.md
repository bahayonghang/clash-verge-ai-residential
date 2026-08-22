# 实施计划：数值条件

## 启动前门禁

- [x] 父规划已批准。
- [x] 本子任务已 `task.py start`。
- [x] 若 `query.rs` 已被 header-sort 修改，先合入再改 `matches_clause`。

## 执行顺序

1. 换算纯函数与 TS 测试：KiB、分钟、负数、溢出。
2. `matches_clause` 数值分支与 Rust 测试：`eq` 0 命中下载 0；`None` 速率不命中；`gt` 不走 contains。
3. `FILTER_FIELDS` 与条件行 UI；字段切换重置。
4. i18n：比较运算与单位。
5. 确认 8 条上限、空值忽略、家宽 AND。

## 验证

前端四门 + `c2::query` 条件测试。不跑 `tinstall`。

## 回滚

去掉数值字段与 mode 分支；条件行回到文本-only。
