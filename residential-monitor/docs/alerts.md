# 告警

- 应用内告警中心是权威记录。Windows 通知是尽力送达。
- 速率：60 秒滚动平均，连续 3 次才触发，恢复使用滞回。
- 周期用量只复用 C3 ReportService / rollup。能力不足时观测值为空，不是零。界面把 `not-evaluable` 与「无告警」分开；`observedValue` 为 null 时显示「未知」。
- 缺口不是零速率。
- 测试通知不写告警历史。
- 安装态普通用户通知与 Focus Assist 真机验收尚未执行。
