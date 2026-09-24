# 实施计划

状态：planning。用户已要求建任务并继续优化，但本文件尚未经过实施批准。不要在批准前改产品代码。

## 顺序

1. 在同一 A250 库的 `[1787184000, 1787184060)` 上拆分未过滤投影与分钟扫描。只读，不改 deadline。
2. 投影占主导时，把 `load_sessions` 限制到窗口内 session，并补 oracle。无分类过滤和家宽过滤都要覆盖。
3. 用新身份重测 1 分钟首读，以及 A250 30 天 host/network 各 21 次。
4. 只有阶段计划显示 A1000 有可能进 10 秒时才生成 A1000。否则保持不生成。
5. 重建或定位 baseline bench。不能重建就保持 AC7 主场景未通过。
6. 同窗口复测 F1–F5。仍超过 1.10 再做摄取阶段归属，归属前不改调度器。
7. F6–F12 按 AC4 收口。空窗口首读不单独触发 SQL 改动。
8. `just ci`。不安装，不提交安装包，不打开自动删除。

## 验证

```text
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib raw_fold::
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c3::service::
just ci
```

容量与矩阵命令沿用前序 `research/benchmark-harness.md`，输出必须在新的隔离目录，且不覆盖已有 JSON。

## 风险

- 跨日矩阵样本可能是机器噪声。先复测，再改代码。
- 窗口内 session 过滤如果漏掉跨分钟会话，会改变 distinct 和排名。oracle 必须包含跨窗口会话。
- baseline exe 缺失时，不能用归档 JSON 的 CPU 宣称主场景通过。
