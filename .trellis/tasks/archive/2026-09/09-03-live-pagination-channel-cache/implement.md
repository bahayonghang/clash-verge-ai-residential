# 实现：实时分页与 Channel 缓存

1. reducer 去掉 `Map<identity, LiveConnectionView>`，改为 `Set` 或只在 closeMarks 逻辑里比较 id 列表。
2. `useLivePage` 增加 `cursor` state 与 `loadNext`/`loadPrev`。
3. Live 页工具条显示「第 n 页 / 匹配数」。
4. 测试：cursor 非空时 invoke 参数含 cursor；matchedCount>limit 仍显示总数。
5. 验证：`npm --prefix residential-monitor test`。
