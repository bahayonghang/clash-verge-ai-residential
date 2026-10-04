# T05 正式计量结果（rebuild-20261002-4f8a8c68）

日期：2026-10-03，America/Chicago；执行 UTC 2026-10-04T01:50:46Z–02:51:22Z。用户确认 ferrots 负载结束后发出 GO。启动前固定机制扫描未匹配竞争进程，CPU 占用 14.1%；matrix、primary、capacity 输出目录启动前均不存在。三阶段串行执行，未并行 quiet-probes 或构建。

## 执行与退出

| 阶段     | native 次数 | native exit | 校验错误 | 竞争负载 | driver exit | 含义                   |
| -------- | ----------- | ----------- | -------- | -------- | ----------- | ---------------------- |
| matrix   | 22/22       | 全部 0      | 0        | 无       | 2           | 有门 FAIL              |
| primary  | 6/6         | 全部 0      | 0        | 无       | 3           | 无 FAIL，有 UNVERIFIED |
| capacity | 106/106     | 全部 0      | 0        | 无       | 3           | 无 FAIL，A1000 NOT_RUN |

三个 driver stderr 均为 0 字节。driver 退出码由外层 shell 直接采集，见 `*-driver.receipt.json`。

## matrix（F1–F12，限值 1.10）

| 门  | 场景 / 指标                          | baseline | candidate | 比值   | 结果 |
| --- | ------------------------------------ | -------- | --------- | ------ | ---- |
| F1  | a1000-metadata ingest p95 ms         | 39.0584  | 43.3910   | 1.1109 | FAIL |
| F2  | a250-backlog ingest p95 ms           | 17.9597  | 19.9013   | 1.1081 | FAIL |
| F3  | a50-unchanged ingest p95 ms          | 13.8272  | 12.4981   | 0.9039 | PASS |
| F4  | a250-failed ingest p95 ms            | 19.4666  | 21.4518   | 1.1020 | FAIL |
| F5  | a250-metadata ingest p95 ms          | 18.7829  | 19.1060   | 1.0172 | PASS |
| F6  | a1000-metadata native private p95 B  | 11530240 | 13275136  | 1.1513 | FAIL |
| F7  | a1000-unchanged native private p95 B | 10698752 | 12492800  | 1.1677 | FAIL |
| F8  | a50-metadata SQLite xWrite B         | 1483200  | 2111360   | 1.4235 | FAIL |
| F9  | a250-failed SQLite xWrite B          | 3232096  | 3561216   | 1.1018 | FAIL |
| F10 | a1000-metadata SQLite xWrite B       | 7880872  | 9288904   | 1.1787 | FAIL |
| F11 | a250-backlog native CPU s            | 0.171875 | 0.046875  | 0.2727 | PASS |

F12 空窗口首读 p95：11 个场景中 6 个 PASS，5 个 FAIL（a50-unchanged 1.1609、a250-counters 1.1652、a250-metadata 1.2209、a1000-unchanged 1.2131、a250-backlog 1.4213）。F12 非空生产 oracle 为 UNVERIFIED。

F8–F10 的比值与 2026-09-30 retained 身份的 matrix 完全相同；SQLite 写入字节在两次身份间不变。F2 由 1.9233 降为 1.1081，F6/F7 由 1.1368/1.1066 升为 1.1513/1.1677。baseline 为 c278 加仪表，candidate 为当前源码；差异的产品原因未查明，不归因到单一改动。

## primary（AC7，三轮合计）

| 门                      | baseline  | candidate | 比值   | 限值 | 结果           |
| ----------------------- | --------- | --------- | ------ | ---- | -------------- |
| CPU 秒                  | 25.671875 | 0.234375  | 0.0091 | 0.70 | PASS           |
| SQLite xWrite B（子集） | 70595256  | 32718048  | 0.4635 | 0.50 | PASS（仅子集） |
| 全部应用文件写入        | null      | null      | —      | 0.50 | UNVERIFIED     |

逐轮 candidate CPU 为 0.09375 / 0.109375 / 0.03125 秒，baseline 为 4.515625 / 10.3125 / 10.84375 秒。数值均为 1/64 秒整数倍。10-01 记录也出现 candidate CPU 为 0.0 的轮次。CPU 门按 runner 判定为 PASS；candidate CPU 计量偏低的原因未查明。

## capacity（10 000 ms 门）

| 场景             | 通过 / 要求     | 最大 wall ms | 结果 |
| ---------------- | --------------- | ------------ | ---- |
| a50-host-30d     | 21/21           | 1788.8       | PASS |
| a50-host-1d      | 21/21           | 89.8         | PASS |
| a50-network-30d  | 1/1（诊断单次） | 1212.0       | PASS |
| a250-host-30d    | 21/21           | 7132.2       | PASS |
| a250-host-1d     | 21/21           | 247.0        | PASS |
| a250-network-30d | 21/21           | 6170.5       | PASS |

所有 native 查询都在 preflight 读库之后执行，没有物理冷页证明。A1000 仍为 NOT_RUN。10-02 在 ferrots 负载下的 A250 30 天生产诊断 `DeadlineExceeded`（10035.87 ms）保留；本次无负载正式门最大 7132.2 ms。两者条件不同，不互相覆盖。

## AC 状态

- AC2：capacity 的精确 oracle 比对通过；跨窗口 session、取消与 deadline 的完整条款没有在本轮单独验收。保持未勾选。
- AC3：A50/A250 所选门通过；A1000 NOT_RUN，冷页条件未分开报告。保持未勾选。
- AC4：F1、F2、F4、F6–F10 与 F12 的 5 个空窗口场景 FAIL，F12 非空 UNVERIFIED。保持未勾选。
- AC5：CPU 与 SQLite 子集通过，全部应用文件写入 UNVERIFIED。保持未勾选。
- AC6：未在本轮验收。保持未勾选。

## 收据 SHA256

| 文件                         | SHA256                                                             |
| ---------------------------- | ------------------------------------------------------------------ |
| matrix/summary.json          | `9611ca68e13ef327c36a3dd3fb100f22a0f8783669187f3e81821f2f1b7bf94a` |
| primary/summary.json         | `8c894f54d99274071c144ce1e5eaeaeaf6b8adb3674c66a1da6ea0e8217c6a1b` |
| capacity/summary.json        | `39f543c538a401e22db52a82625ee6d629d081dcd8f4b8ffbb1597310013751f` |
| matrix-driver.receipt.json   | `978504236f6316b53d24d16c91d73b553d7d564eea62316b8601f84596b44914` |
| primary-driver.receipt.json  | `a8fec7814cf125c86d85f27cd74e96243684b08be8dff8504d9c7700fb0e0fae` |
| capacity-driver.receipt.json | `5c414f1d397866770d1eae4f1a6076f34a29bde8552b01ce35a7d1af948c7b0e` |

本轮未修改产品源码、runner、语料或门槛。未提交、未归档。
