# DTO 解码

- Rust 是权威校验者。前端解码失败时显示专门中文状态，不猜测缺字段。
- 每条 Channel 消息必须检查 `schemaVersion`、`kind` 和单调 `seq`。
- 禁止把 mihomo 原始 JSON 或 SQL 行传到视图层。
- 时间展示用用户本地时区；持久时间保持 UTC integer。
