import type { ReactNode } from "react";
import { cn } from "../../lib/utils";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";

export function OverviewCard({
  title,
  icon,
  action,
  footer,
  children,
  className
}: {
  title: string;
  icon: ReactNode;
  action?: ReactNode;
  footer?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <Card
      className={cn(
        "flex h-full flex-col overflow-hidden border-border/50 bg-gradient-to-b from-card to-card/50",
        className
      )}
    >
      <CardHeader className="shrink-0 space-y-0 p-4 pb-3">
        <div className="flex items-center justify-between gap-2">
          <CardTitle className="flex items-center gap-2 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
            {icon}
            {title}
          </CardTitle>
          {action ? <div className="flex items-center">{action}</div> : null}
        </div>
      </CardHeader>
      <CardContent className="flex flex-1 flex-col px-4 pb-4 pt-0">
        <div className="flex-1">{children}</div>
        {footer ? <div className="mt-auto border-t border-border/30 pt-4">{footer}</div> : null}
      </CardContent>
    </Card>
  );
}
