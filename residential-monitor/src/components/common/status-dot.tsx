import { cn } from "../../lib/utils";
import { healthTone, type HealthTone } from "../../lib/health";
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";

const TONE_CLASS: Record<HealthTone, string> = {
  ok: "bg-emerald-500",
  warn: "bg-amber-500",
  bad: "bg-rose-500"
};

const PING_CLASS: Record<HealthTone, string> = {
  ok: "bg-emerald-400/70",
  warn: "bg-amber-400/70",
  bad: "bg-rose-400/70"
};

export function StatusDot({
  session,
  label,
  ping = true
}: {
  session: string;
  label: string;
  ping?: boolean;
}) {
  const tone = healthTone(session);
  const busy = ping && (tone === "warn" || tone === "bad");
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span
          className={cn("relative inline-flex shrink-0", busy ? "h-3 w-3" : "h-2.5 w-2.5")}
          role="status"
          aria-label={label}
        >
          {busy ? (
            <span
              className={cn(
                "absolute inline-flex h-full w-full rounded-full animate-ping",
                PING_CLASS[tone]
              )}
            />
          ) : null}
          <span
            className={cn(
              "relative inline-flex h-full w-full rounded-full",
              TONE_CLASS[tone]
            )}
          />
        </span>
      </TooltipTrigger>
      <TooltipContent side="bottom">
        <p>{label}</p>
      </TooltipContent>
    </Tooltip>
  );
}
