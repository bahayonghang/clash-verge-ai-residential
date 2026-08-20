# 实施：方向热点摘要

1. [ ] 在 Rust query 中定义轻量 summary/hotspot DTO，补 matched 全量与稳定 tie-break 测试。
2. [ ] 从同一 hub snapshot 暴露 sampleUtc/summary，更新 command response 与 TS 类型/decoder。
3. [ ] 接入 applied query 的 refresh/render，添加两张方向卡和状态/本地化/无障碍文案。
4. [ ] 补缺字段/null/无匹配/暂停/缺口/过期响应/防泄漏测试；运行前端 check、Rust query tests、fmt。

依赖：筛选子任务先冻结 applied query 交互；不在本子任务中修改 collector、Channel message 或数据库。
回滚点：保留旧 rows/nextCursor 读取路径，summary 失败只隐藏/标记卡片，不回退到前端 Top 1。
