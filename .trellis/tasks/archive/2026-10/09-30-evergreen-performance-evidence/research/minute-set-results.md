# MinuteSet 实验处置

日期：2026-09-30。适用工具：Claude Code、Codex、Grok Build、Kimi Code、OMP。

状态：强模型主会话与独立审查同意回退连续重复分钟缓存。保留 lazy 字典、chain 缓存、network 合同修复、独立集合语义测试及全部实验收据。

## 语义与资源

- 原实现新增两项测试通过；`MinuteSet` 实测 96 B，`Option<i64>` 为 16 B。
- 实验实现 raw-fold 7 passed / 1 ignored；`MinuteSet` 为 112 B。独立 HashSet oracle 覆盖连续/非连续重复、乱序、负值、i64 极值、dense padding 和无效输入。
- 重复分钟不跳过字节或 session 累加；fixture 仍得到 upload=18、download=23、connections=4、duration=120 s。
- 回退只去除 `last` 字段、初始化、短路、更新和对应内部状态断言。独立集合及累加测试保留。

## 两对局部结果

固定 before：`projection-cache-lazy-20260930` test SHA256 `EFBD43FFB61FC8D0CB290F116AC6F0FB6A5888C000332FC4C4E093429D3E37DC`。实验 candidate：`candidate-final-20260930` test SHA256 `03A36CF38C8FB8F0651C82F07FCE1BA606350E684CBAE3CE3A2630CE54E80FBF`。

相同 A250 30 天库；数据库页缓存已被之前的 probe/oracle/hash 读取。两对交替顺序：before→candidate、candidate→before。all 使用本地时区及 previous window；residential 使用 UTC，无 previous window。只做同筛选前后比较。

| 项目 | before 中位 ms | candidate 中位 ms | candidate/before |
| --- | ---: | ---: | ---: |
| all projection | 2756.11115 | 2763.51160 | 1.002685 |
| all scan | 3843.77135 | 3792.05645 | 0.986546 |
| residential projection | 2263.44440 | 2430.01310 | 1.073591 |
| residential scan | 3445.22220 | 3398.32080 | 0.986387 |

30 天 all 默认生产报告单次 6696.6586→6684.6417 ms，下降 0.18%；精确 totals 与 720 个 series 行一致。一分钟生产报告 4.6843→4.8528 ms，两端精确 oracle 通过。所有 native test exit=0、实际运行 1 个测试、JSON 存在。

扫描约 1.35% 的名义下降与单次整报告 0.18% 的变化不足以证明稳定收益。增加每集合 16 B 与额外状态不予保留。residential projection 的反向变化原因未查明，不直接归因于 MinuteSet。未增加样本追求微小收益。

## 身份及验收边界

`candidate-final-20260930` 是已退出的 MinuteSet 实验身份，不能用于最终生产候选验收。目录、exe、完整 just ci、release 检查和比较收据全部保留。保留方案将使用 `candidate-retained-20260930` 的新身份及重跑门。

本局部实验不关闭 F1–F12、AC7、完整 A50/A250/A1000、21 次、安装态或 WebView 门。正式矩阵尚未启动。

证据：`minute-set-20260930/`、`candidate-final-20260930/`、`minute-set-comparison-20260930/`。
