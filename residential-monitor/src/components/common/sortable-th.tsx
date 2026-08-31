import { ChevronDown, ChevronsUpDown, ChevronUp } from "lucide-react";
import { cn } from "../../lib/utils";

export type SortAria = "none" | "ascending" | "descending";

export function SortableTh({
  label,
  ariaSort,
  onClick,
  numeric = false,
  subtle = false,
  className
}: {
  label: string;
  ariaSort: SortAria;
  onClick: () => void;
  numeric?: boolean;
  /** 对齐共享表格表头档差（text-xs font-medium），默认保持原样避免外溢。 */
  subtle?: boolean;
  className?: string;
}) {
  const Icon =
    ariaSort === "descending" ? ChevronDown : ariaSort === "ascending" ? ChevronUp : ChevronsUpDown;
  return (
    <th
      className={cn(
        subtle ? "py-2 text-xs font-medium" : "py-2 font-semibold",
        numeric ? "px-2 text-right" : "text-left",
        className
      )}
      aria-sort={ariaSort}
    >
      <button
        type="button"
        className={cn(
          "inline-flex items-center gap-1 hover:text-foreground",
          numeric && "ml-auto",
          ariaSort === "none" ? "text-muted-foreground" : "text-foreground"
        )}
        onClick={onClick}
      >
        <span>{label}</span>
        <Icon
          aria-hidden="true"
          data-sort-icon={ariaSort}
          className={cn(
            "size-3.5 shrink-0",
            ariaSort === "none" ? "text-muted-foreground/80" : "text-foreground"
          )}
        />
      </button>
    </th>
  );
}
