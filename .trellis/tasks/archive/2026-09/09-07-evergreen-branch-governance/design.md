# Design

Target：GitHub repo `bahayonghang/clash-verge-ai-residential`，branch `main`，既有branch-protection合同。

Files:
- `.trellis/spec/frontend/quality-guidelines.md` 的Main Branch Protection段：仅记录批准结果/验证日期，不假装本地文件即远端设置。
- 本子任务 `research/protection-before.json` / `protection-after.json` / `validation.md`：获批准执行时保存脱敏API证据；本轮不预建伪快照。

批准前准备完整可审查请求，基于新鲜GET转换到支持的PUT请求schema；不能把GET响应原样PUT，也不能用不完整payload清空其他字段。目标只改变 required_linear_history；发送前再读一致性，状态有变化则重新准备差异。该步骤应由熟悉API的强模型负责。

回滚：若出现由本项导致的非目标漂移，只在已批准恢复原快照范围内修正并GET验证；不得绕过管理权限或修改历史。
