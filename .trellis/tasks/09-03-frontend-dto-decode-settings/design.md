# 设计：严格解码与设置序号

## 边界

`dto.ts`、`ipc/decoder.ts`、`ipc/live-session.ts`、`hooks/use-settings.ts`、`hooks/use-alerts.ts`、`hooks/use-report-archive.ts`、家宽 `report-section.tsx`。Rust 仍是权威校验。

## 契约

每个 DTO 解码器：own-property + 类型；失败 throw。`processPath` 即使存在也丢弃。`LiveConnectionView` 类型删除该字段。

`useSettings`：`secretSeq` / `connectionSeq` / `aboutSeq` / `dataSeq` 四个计数器，仿 autostart。

`renderReportHtml` 只从 `useReportArchive` 调用。
