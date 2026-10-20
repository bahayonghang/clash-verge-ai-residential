# 修复与本机应用验收

日期：2026-10-19（America/New_York）。用户批准实施验证和本机处理，明确不提交。

## 结果

仓库脚本已修复顶层 null 兼容；非法类型、同名规则集所有权和引用冲突校验保留。公开脚本运行代码仅增加 3 行。独立代码审阅无问题，详见 [check-review.md](check-review.md)。完整质量门最终结果见下表。

本机已定位选中的“家宽”扩展脚本，先备份，再应用与公开脚本相同的 clone 归一补丁。替换前逐字比对 clone helper，唯一差异是待添加的 3 行；修补后其余字节、原凭据、开关均保留。未替换整份模板，未手改仓库 local TOML/JS，未改订阅文件或数据库。

## 公开自动化证据

| 验证 | 状态 |
| --- | --- |
| 独立旧模板对四个 null 正例，修复前及修复后再次执行 | 两次 RED：4/4 因目标类型错误失败，exit 1 |
| 新模板 null/正则 DNS 窄测试 | PASS：16/16 |
| renderer 测试 | PASS：26/26；后补家宽断言也由最终 root CI 覆盖 |
| 根 npm run ci | PASS：259/259，语法、agent contract、secret scan 通过 |
| just ci | PASS：exit 0；monitor 前端 73 文件/300 tests，Rust workspace 550 + 3 tests 通过、6 ignored；fmt/clippy、root 259 tests、secret scan 全部通过 |
| 独立 check | PASS：未发现代码或测试问题，无自修扩权 |
| git diff --check | PASS，最终再次核对 |

红绿用例、旧/新公开模板 SHA 和命令见 [implementation-evidence.md](implementation-evidence.md)。旧研究 repro.cjs 固定了修复前拒绝 null 的预期，仅为历史诊断资料，不能当本次绿测入口。

## 本机修改与实机观察

- 安装文件版本：FlClash `0.9.1+2026101702`。
- 已选脚本“家宽”由报错订阅关联，本机存储脚本已通过 node --check。
- 原脚本备份位于本机 `%LOCALAPPDATA%/Codex/FlClashBackups/20261020T011719Z/`；该目录含原始脚本及本地回退 receipt，原内容与凭据不复制到仓库。
- 私有脚本修复前 SHA-256：`cbe6764cbcc0e3d832d93a4dfd2d475a2aa9e0f0d4905094de407085000f7d20`。
- 私有脚本修复后 SHA-256：`aa61798ed43ad50596aee977c1d9313e6918d8f6d3af6363e04f0c4f50cb3fd8`。
- 操作前用户停留在正常的另一份标准订阅；修补完成后，通过真实窗口重新选择原报错订阅，未故意在旧脚本下再次打断当前代理。
- 目标订阅 YAML 没有声明 rule-providers。修复后目标订阅实际应用成功，保持 rule 模式，侧栏“代理”可见，未出现目标类型错误红框。
- 实际“代理”页可见 AI-家宽、家宽-SOCKS5、节点选择、自动选择及原机场其它业务分组；不是单独移除错误文字。
- 运行 YAML 统计：原节点 73 → 74，原组 11 → 12；原有节点名称和组名均为输出子集，新增家宽节点及组；专用正则 DNS provider 存在。脱敏结果见 [native-validation-summary.json](native-validation-summary.json)。
- 当前保持在修复后的目标订阅和代理页，便于用户继续使用。原先选择记录在本地备份 receipt，可用于恢复；没有删除脚本或订阅。

## 验收映射与边界

F-AC1–F-AC5 由自动化正反例覆盖；F-AC6 的完整门状态以上表为准；F-AC7 的本机正向应用已执行通过。没有在用户的正常运行配置上重新加载旧脚本制造故障；旧版失败由用户截图及独立红测支撑。

未捕获 FlClash 报错当时传给 main 的实际对象；null 的确定性回归与客户端修复后应用成功是两条独立证据。遗留 config.json 仍对应另一订阅，未将其类型/节点数当成本次目标输入证明。未运行 Node 18/20/22 平台矩阵，未测试住宅出口纯净度、公网 DoH、真实 SOCKS5 连通或 UDP；本次结论限于目标输入错误和配置可应用性。

## Git 与任务

未提交、未推送，不执行会自动提交的 archive/finish-work 路径。任务保持 in_progress 供用户审阅未提交改动。产品范围仅批准的五文件；任务证据与必要的 Trellis 会话记录另列。禁止将本机原脚本、备份或订阅内容加入 Git。

## 独立依赖告警

monitor 的 npm ci 报 1 项 high；额外只读 npm audit 确认传递依赖 source-map-js（1.0.0–1.2.1）受 [GHSA-68fv-2mgg-jv7q](https://github.com/advisories/GHSA-68fv-2mgg-jv7q) 影响，audit 退出 1，详见 dependency-audit-warning.json。未执行 audit fix，未改依赖或锁文件。该独立审计未通过，不能由 just ci 的 PASS 覆盖；本次未执行完整 just dependency-audit（docs npm/RustSec）。

会话收尾脚本调用被自动审批策略拦截，未执行；改以本任务资料保存全部结果，未请求或进行任何提交。

## 提交与归档授权更新

用户已批准本地提交本修复和全部相关 Trellis 任务资料，并归档任务；禁止推送，依赖告警暂不处理。上文“未提交/不归档”和审批拦截描述是上一实施轮的历史状态，本轮按新授权收尾；不将本机原脚本、备份、凭据或订阅加入提交。

归档结果：修复及研究资料已提交 `38b6fe0`，任务已移入 `.trellis/tasks/archive/2026-10/10-20-flclash-script-provider-failure`，状态 completed。归档时仅更新任务指针、归档脚本相对根路径及收尾记录；代码未再修改，依赖未处理，未推送。
