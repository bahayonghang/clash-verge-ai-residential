# 实现：严格解码与设置序号

1. 补 `decodeLiveRow` / `decodeAlertRule` / `decodeTraySummary`。
2. 剥 `processPath`。
3. 拆 `use-settings` 序号；secret catch 设 `errorZh`。
4. HTML 导出移入 archive hook；扩展 `ipc-boundary.test.ts`。
5. 验证：`npm --prefix residential-monitor test` 与 `typecheck`。
