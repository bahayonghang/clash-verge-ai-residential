# C5 验收证据

候选提交基线：C4 `e83cc5c`。工作树含本任务改动。未发布 Release。

| 项 | 命令或文件 | 结果 |
|---|---|---|
| C5-AC1 依赖完整 | 归档 C0–C4 gate / AC 文件 | 通过追溯。C4 自动门通过，但 C3-AC10 / C4-AC7 / C4-AC9 仍有未完成项，C5 不得升级为发布容量 |
| C5-AC2 安装态端到端 | 未执行 `just tinstall` | 未通过 / 暂停。需再确认后才改本机安装态 |
| C5-AC3 视觉无障碍 | `src/styles.css`、`src/main.ts`、`src/ipc/routes.test.ts`；HTML `@media print` | 自动检查通过。Windows 真机键盘 / 高 DPI / 安装态走查未做 |
| C5-AC4 故障矩阵 | `monitor-bench c5-fault`；`research/evidence/c5-fault.json` | fixture 7 项通过。控制器真机、睡眠 / 重启、Focus Assist 未做 |
| C5-AC5 手动升级 | `monitor-bench c5-baseline` 退出码 2 | 未通过。C0 安装包与 schema fixture 缺失 |
| C5-AC6 签名与资产 | `c5-supply`；未签名例外模板 | 当前 installer SHA-256 已记录，`signed=false`。无证书，无负责人签字例外，无 Release draft |
| C5-AC7 卸载语义 | `c5::purge` 测试；文档 `upgrade-uninstall.md` | 自动化删除清单与部分失败文案通过。普通卸载真机未做。未写 Credential Manager |
| C5-AC8 性能数据集 | 未重跑完整 30 天三档 / 13 个月库 | 未通过 / 暂停。不得沿用 C0 生成摘要当 C5 重跑 |
| C5-AC9 指标与门限 | 无新的完整规模原始分布 | 未通过。C0 peak max=3321ms 已超过 3s 正常最大值，C5 未重跑 |
| C5-AC10 并发与取消 | `c5-concurrent` fixture | fixture 通过。不是 30 天并发门 |
| C5-AC11 24 小时稳定性 | `c5-soak-smoke`；`full24h=false` | 未通过 / 暂停。日程已冻结，24 小时未跑 |
| C5-AC12 低空间 fail closed | `c5::vacuum`、`c5-fault` backup / restore | fixture 通过。VACUUM 中段注入与 30 天库未做 |
| C5-AC13 文档与供应链 | `residential-monitor/docs/`、`c5-supply`、`secretHits=[]` | 文档与 lock 扫描通过。安装态文档实走未做 |
| C5-AC14 独立发布 Gate | 本文件 + `c5-go-nogo.md` | **no-go**。自动质量门另见 `just monitor-check` / `just ci` |

## 暂停条件已触发

- C0 升级基线安装包与 schema fixture 缺失。
- 完整 30 天 `A=50/250/1000`、13 个月 rollup、24 小时 soak 环境/时间不够。
- 即将写入本机 NSIS 安装、登录自启动、Credential Manager 或系统通知，尚未再确认。
- 未签名、未创建 GitHub Release。
