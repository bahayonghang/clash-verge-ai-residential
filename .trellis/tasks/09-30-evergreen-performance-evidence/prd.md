# T05 恢复未完成性能门的可重复输入和基线，定位长窗口报告与矩阵问题，再按证据实施最小改造。

## Goal

恢复未完成性能门的可重复输入和基线，定位长窗口报告与矩阵问题，再按证据实施最小改造。

## Evidence

F6；短窗口已有修复，长窗口、同窗口矩阵和 AC7/AC8 完整证据不足。 详细证据见 ../09-30-evergreen-five-harness-audit/research/audit.md。

## Requirements

- R1 解决本任务证据对应的问题，不扩大到相邻清理。
- R2 按下面可观察结果验收，保留失败与未验证边界。
- R3 实施后将批准结果回写并注明适用工具。

## Acceptance Criteria

- [x] AC1：重建或找到 baseline 的源码、构建参数、工具链和 exe hash；无法重建则明确保留同窗口门未过，不能用历史 JSON 替代。2026-09-30 重建与独立身份审查通过，见 research/review.md、research/review-build-identity.json。
- [ ] AC2：相同生成语料、窗口、分组与过滤下，短窗口及跨窗口 session 的 totals/distinct/rank/coverage 与 oracle 相同，取消和整个报告 deadline 保持。
- [ ] AC3：A50/A250/A1000 完整 30 天所选正式容量门逐项记录；A250 host/network 与 A1000 host 的既定 21 次全部 exit 0，不超过 10 秒，冷/热页缓存条件分别报告。
- [ ] AC4：F1–F12 逐项按同窗口对照验收；变化场景 p95 与 native private 比值 <=1.10；F8–F10 保留 SQLite xWrite 比值 <=1.10，F11 按原任务「同一规则收口」保留 CPU 比值 <=1.10；非空和空窗口首读分开，原因未明不得销项。原始条款与各场景位置见 research/measurement-prerequisites.md。
- [ ] AC5：AC7 保持 A250/1Hz/metadata 不变/档案齐全，30 秒预热后 3 轮各 300 秒；CPU 秒下降 >=30%，归属应用文件写入下降 >=50%。SQLite 子集不得代替全部应用文件门。
- [ ] AC6：AUTO_DELETE_ENABLED=false、WAL/FULL、schema 和核算语义不变。AC8 的 10k/1Hz/30分钟、24h 安装态、WebView/后台 worker 证据独立记录，未跑仍未通过。

## Dependencies

初始测量设计可独立进行；正式对照不得与构建/其它负载并行。源码改动前必须确认 benchmark 身份和阶段归属。

## Authorization

用户于 2026-09-30 明确批准项目内实施。具体文件及步骤见 design.md / implement.md。全局配置、真实凭据/数据库、安装、push、PR、远端 workflow、提交与归档不在默认授权内。

## 2026-10-03 正式计量

matrix 22/22、primary 6/6、capacity 106/106 native 全部 exit 0，无竞争负载。matrix F1/F2/F4/F6–F10 与 F12 的 5 个空窗口场景 FAIL；primary CPU 与 SQLite 子集 PASS，全部应用文件 UNVERIFIED；capacity A50/A250 全部 PASS，A1000 NOT_RUN。AC2–AC6 仍不满足完整条款，未勾选。见 research/rebuild-20261002-4f8a8c68/formal-result-20261003.md。

## 2026-10-04 正式计量（方向 1 与方向 2 实施后）

身份 rebuild-20261004-e14ecd88。matrix 22/22、primary 6/6、capacity 106/106 native 全部 exit 0，无竞争负载。matrix F1–F4、F8、F11 PASS；F5、F6、F7、F9、F10 与 F12 的 9 个空窗口场景 FAIL，F12 非空 UNVERIFIED。primary CPU 0.0242 与 SQLite 子集 0.4636 PASS，全部应用文件 UNVERIFIED。capacity A50/A250 全部 PASS（A250 30 天最大 6917.9 ms），A1000 NOT_RUN。隔离回收容量 A250 first-day 完成（7455 块，748.3 s，单块最大 433.9 ms）；A50 用尽 100000 块未完成，原因是 b8a64a1 引入的字典与覆盖游标相位锁定；用户批准后修复 `cleanup_expired`，复测 A50 完成（2511 块，166.8 s，单块最大 377.9 ms），A250 完成（7433 块，682.6 s，单块最大 419.6 ms）。heap 诊断显示 F6/F7 差额不在 Rust 堆或 SQLite 峰值中，来源原因未查明。AC2–AC6 仍不满足完整条款，未勾选。见 research/rebuild-20261004-e14ecd88/formal-result-20261004.md 与 research/f6-f10-attribution-20261004/。
