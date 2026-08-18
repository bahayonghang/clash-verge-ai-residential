# C5 Gate 状态

生成时间：2026-08-18。

| Gate | 结果 | 证据 |
|---|---|---|
| 1 候选与证据索引 | 部分 | `c5-evidence-index.md`。C0 基线资产缺失 |
| 2 跨层集成审查 | 通过 | `c5-cross-layer-review.md`。未退回 C1–C4 改语义 |
| 3 视觉无障碍 | 部分 | 状态文案、焦点恢复、skip link、打印 CSS。真机走查未做 |
| 4 故障矩阵 | 部分 | `c5-fault` 7/7 fixture 通过。真机生命周期未做 |
| 5 最终规模数据集 | 未通过 | 未重跑三档 30 天库与 13 个月高基数 |
| 6 10,000 短峰 | 未通过 | 未重跑 10k×30 分钟。不把 C0/C4 fixture 升级为发布容量 |
| 7 并发门 | fixture | `c5-concurrent.json`。scale=`fixture` |
| 8 低空间 | fixture | vacuum / backup / restore fail closed 测试通过 |
| 9 24 小时 soak | 未通过 | `c5-soak-smoke.json` `full24h=false` |
| 10 NSIS 安装升级卸载 | 未通过 | 未执行 `tinstall`。C0 基线缺失 |
| 11 签名与供应链 | 部分 | lock 扫描无 secret。installer SHA-256 已记录。未签名、无例外签字 |
| 12 文档与 go/no-go | no-go | `c5-go-nogo.md` |

## 自动质量门

| 命令 | 退出码 |
|---|---|
| `just monitor-check` | 0 |
| `just ci` | 0 |
| `cargo test --lib c5::` | 0（11 passed） |
| `monitor-bench c5-baseline` | 2（C0 基线缺失，预期失败） |
