import type { ComponentProps, ReactNode } from "react";
import { cn } from "../../lib/utils";

/**
 * 共享表格规格（DESIGN.md Data table 条目）：
 * 数值列右对齐 + tabular-nums，表头与正文拉开档差，行 hover 统一。
 * 仅收敛 class，不持有排序/筛选状态（功能由调用方持有）。
 */
export const dataTableClasses = {
  wrapper: "overflow-x-auto",
  /** 不无脑 w-full 拉伸；需要均分时由调用方自行改 w-full table-fixed。 */
  table: "w-auto min-w-0 max-w-full text-sm",
  headRow: "border-b border-border/60 text-left",
  th: "py-2 pr-6 text-xs font-medium text-muted-foreground last:pr-0",
  thNumeric: "py-2 pl-6 text-right text-xs font-medium text-muted-foreground",
  row: "border-b border-border/40 transition-colors last:border-0 hover:bg-muted/40",
  td: "py-2 pr-6",
  tdNumeric: "py-2 pl-6 text-right tabular-nums whitespace-nowrap",
  emptyCell: "py-3 text-muted-foreground"
} as const;

export function DataTableTh({
  numeric = false,
  className,
  ...props
}: ComponentProps<"th"> & { numeric?: boolean }) {
  return (
    <th
      className={cn(numeric ? dataTableClasses.thNumeric : dataTableClasses.th, className)}
      {...props}
    />
  );
}

export function DataTableTd({
  numeric = false,
  className,
  ...props
}: ComponentProps<"td"> & { numeric?: boolean }) {
  return (
    <td
      className={cn(numeric ? dataTableClasses.tdNumeric : dataTableClasses.td, className)}
      {...props}
    />
  );
}

/** 空态 / loading 行的固定写法。 */
export function DataTableEmptyRow({
  colSpan,
  children
}: {
  colSpan: number;
  children: ReactNode;
}) {
  return (
    <tr>
      <td className={dataTableClasses.emptyCell} colSpan={colSpan}>
        {children}
      </td>
    </tr>
  );
}
