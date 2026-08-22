# 覆盖与尾差

历史是带覆盖说明的观测下界。

- 应用启动前、断线、睡眠、暂停、存储故障和退出形成显式覆盖区间。
- 连接消失后的尾部流量不能按连接补回。
- 控制器 meter 与 attributed observed 分开展示。
- `unattributed_gap` 与 `over-attributed` 按同一 frame / epoch / 方向单独记录。负差不抵扣历史。

C5 不得把开发态 smoke 或小库线性外推写成发布容量。
