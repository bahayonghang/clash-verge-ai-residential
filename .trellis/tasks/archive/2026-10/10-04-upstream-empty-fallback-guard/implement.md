# 实施计划：空组回退防护

## 前置

- [x] 用户后续批准最新规划后，才 `task.py start`。
- [x] 读取父任务 F2 复现、现有组图 spec 和测试。核对共享文件是否已被其它任务修改。

## 顺序

1. [x] 在 `tests/regression.test.js` 的递归/保留名称区域增加顶层与嵌套住宅 fallback 失败测试，先验证当前实现接受这些输入。
2. [x] 在现有 DFS 中加入保留名称检查；复用 fail/injectedNames，不抽象重构。
3. [x] 增加普通 fallback、缺省、不相关组及未知字段保留测试；每个拒绝用例断言输入不变和错误不含凭据。
4. [x] 更新配置/排错说明、state-management spec、CHANGELOG Unreleased；保留宿主失败回退边界。

## 验证矩阵

| 编号 | 场景                                            | 预期                                      | AC  |
| ---- | ----------------------------------------------- | ----------------------------------------- | --- |
| F1   | 顶层 url-test、筛选无成员、fallback=家宽-SOCKS5 | 拒绝，字段/组/路径明确，输入不变          | AC1 |
| F2   | Proxy→Nested、Nested fallback=家宽-SOCKS5       | 拒绝并显示嵌套路径                        | AC1 |
| F3   | fallback=AI-家宽                                | 拒绝保留出站，不声称该组引用本来有效      | AC2 |
| F4   | 普通机场节点 fallback                           | 保持字段和原配置语义                      | AC3 |
| F5   | 无 fallback、不可达组含该字段、未知字段         | 新检查不扩大影响                          | AC3 |
| F6   | 原 include-all/循环/UDP/默认投影/幂等           | 全部保持                                  | AC4 |
| F7   | 脱敏实际 Profile/内核空组场景                   | 记录内核版本与配置/选择行为；不碰生产路由 | AC5 |

## 命令和边界

- [x] `node --test tests/regression.test.js`
- [x] `just ci`
- [x] `just docs-build`
- [x] 将 F1–F7、实际命令、结果和阻塞写入本任务 `research/validation.md`。

真实检查在隔离、无真实凭据的配置中进行；不向当前生产 controller 推送。环境不可用记 BLOCKED，不安装工具或提高权限绕过。新增行为超出保留名称检查时返回规划。提交/归档/推送另需用户授权。
