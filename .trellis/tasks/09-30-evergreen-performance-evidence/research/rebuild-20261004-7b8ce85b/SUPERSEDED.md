# 已被取代

本目录的 P1（构建与 smoke）与 P2（A50/A250 语料、SQL oracle、4 个探针）均 exit 0，但未运行 matrix、primary、capacity、retention 或 heap 诊断。

P2 结束后，独立静态审查（无 blocker）提出 F2（跳跃枚举每步重新 prepare）、F3（测试缺口）、F4（spec 措辞）、F6c（SQLite 统计读取与重置非原子）。修正后源码与本目录构建身份不一致，正式计量改在 `../rebuild-20261004-e14ecd88/` 进行。本目录与 `bench-data/t05-rebuild-20261004-7b8ce85b/` 保留为记录，不作为任何门的证据。
