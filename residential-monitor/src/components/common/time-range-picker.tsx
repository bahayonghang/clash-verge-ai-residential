import { Clock } from "lucide-react";
import { t, type UiLocale } from "../../i18n";
import { TIME_RANGE_PRESETS, type TimeRange, type TimeRangePreset } from "../../lib/time-range";
import { cn } from "../../lib/utils";
import { Button } from "../ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger
} from "../ui/dropdown-menu";

export function TimeRangePicker({
  locale,
  value,
  onChange,
  className
}: {
  locale: UiLocale;
  value: TimeRange;
  onChange: (preset: TimeRangePreset) => void;
  className?: string;
}) {
  const label =
    value.preset === "today"
      ? t(locale, "time.preset.today")
      : `${t(locale, "time.recent_prefix")} ${t(locale, `time.preset.${value.preset}`)}`;
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          className={cn(
            "h-9 w-[152px] justify-between rounded-xl border-0 bg-secondary/45 px-3 text-sm shadow-none hover:bg-secondary/65",
            className
          )}
        >
          <span className="flex min-w-0 items-center gap-2">
            <Clock className="h-4 w-4 text-muted-foreground" />
            <span className="truncate">{label}</span>
          </span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-44">
        {TIME_RANGE_PRESETS.map((preset) => (
          <DropdownMenuItem
            key={preset}
            onClick={() => onChange(preset)}
            className={preset === value.preset ? "bg-muted" : undefined}
          >
            {preset === "today"
              ? t(locale, "time.preset.today")
              : `${t(locale, "time.recent_prefix")} ${t(locale, `time.preset.${preset}`)}`}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
