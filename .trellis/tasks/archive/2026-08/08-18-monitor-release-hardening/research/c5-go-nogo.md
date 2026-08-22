# C5 go / no-go

结论：**no-go**。不得发布 GitHub Release。

## 已具备

- C0–C4 自动验收可追溯。
- C5 增加关于页、显式删除、用户 VACUUM、故障矩阵入口、并发 fixture、soak 日程、供应链扫描和文档。
- 当前构建 installer SHA-256：`9bbe4b831f6a00b6ce3f34703fe823251423eec57a309db6e7e46a860ded5425`。`signed=false`。
- secret 扫描命中为空。

## 阻止发布

1. C0 冻结 NSIS 基线与 schema fixture 缺失，`c5-baseline` 退出码 2。
2. 完整 30 天 `A=50/250/1000` 与 13 个月高基数未在 C5 重跑。
3. 10,000×30 分钟完整组合未重跑。
4. 24 小时 soak 未执行。
5. 安装态端到端、通知 / Focus Assist / 自启动真机未做。
6. 无 Authenticode 签名，也无针对该哈希的负责人例外。
7. 未创建 Release draft。

## 下一步人工决定

- 找回或重新冻结 C0 基线资产（不得用当前包冒充旧版本）。
- 再确认后才执行 `just tinstall`、登录自启动、Credential Manager 真机和系统通知。
- 安排完整三档库、13 个月库、10k×30 分钟和 24 小时 soak 的机器时间。
- 签名或按 `c5-unsigned-exception-template.md` 批准具体哈希。
