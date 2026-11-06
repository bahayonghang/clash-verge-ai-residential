# 实施计划：正则住宅 DNS

## 前置 gate

- [x] 用户后续批准最新规划后，才运行 `task.py start`。
- [x] 读取父任务三份研究和本任务上下文；确认基线未变化。
- [x] 记录实际 Mihomo 版本。用无真实凭据的独立最小配置验证 inline/classical provider 能被 nameserver-policy 引用，并能匹配 DOMAIN-REGEX。
- [x] 先证明真实 DNS 请求的正负匹配；不要仅运行 `mihomo -t -f <sanitized-config>` 就通过。若缺少内核、权限或可用环境，记 BLOCKED，返回规划，不改用标签内通配或宽后缀。

## 实施顺序

1. [x] 增加正向/负向及开关矩阵测试（AC1/AC2），保留旧 fixture，手工指定允许的默认 DNS 增量。
2. [x] 实现最小 inline provider 生成/所有权检查和 policy 绑定（AC1/R3）。不变更业务路由函数的输出。
3. [x] 添加旧输出撤销、重复执行、输入不变、名称冲突、残留引用和未知配置保留测试（AC4）。
4. [x] 在已有 renderer 测试中验证 Vertex/Cursor 两个开关的生成脚本；测试使用临时虚构 TOML，不接触用户文件（AC5）。
5. [x] 更新 DNS 例外说明、配置文档关联段、相关 spec 和 Unreleased。保留真实运行及宿主失败边界，不宣称所有 DNS/UDP 已同出口。

## 验证矩阵

| 编号 | 场景                                            | 预期                                             | AC      |
| ---- | ----------------------------------------------- | ------------------------------------------------ | ------- |
| D1   | Vertex on / Cursor off                          | 仅 Vertex regex DNS；业务规则不变                | AC1/AC3 |
| D2   | Vertex off / Cursor on                          | 仅 Cursor regex DNS                              | AC1     |
| D3   | 两者 on                                         | payload 为两组现有模式并集，顺序固定无重复       | AC1/AC4 |
| D4   | 两者 off、on→off→on                             | 无新 regex DNS；托管 provider/policy 清理及恢复  | AC4     |
| D5   | 六个负向主机                                    | 均不被新 provider 匹配                           | AC2     |
| D6   | 同名非托管、未知引用、非法 provider 映射        | 中文错误且输入完全不变                           | AC4     |
| D7   | 未知普通 provider/规则、保留未托管 policy 开/关 | 不误删；托管旧键不残留                           | AC4     |
| D8   | renderer 既有开关                               | 临时生成脚本与源模板门控一致                     | AC5     |
| D9   | 私网、bootstrap、AnyRouter、宿主字段            | 原边界保持，默认业务投影不变                     | AC3     |
| D10  | 实际内核/脱敏 Profile 真实查询                  | Vertex 正向/负向、Cursor 开/关的解析出站符合策略 | AC6     |

## 命令与验收

- [x] `node --test tests/regression.test.js tests/sync-local-config.test.js`
- [x] `just ci`（含 monitor gate；不得以根 npm ci 代替）
- [x] `just docs-build`
- [x] 在本任务 `research/validation.md` 记录 D1–D10、版本、命令、实际结果与阻塞。真实验证只保留脱敏信息。

不安装全局工具，不自行读取/修改生产 controller 或用户 local 文件。新授权需求交回用户。前置 gate 不支持设计或测试要求扩大范围时停止并修订规划；提交/推送/发布另需授权。
