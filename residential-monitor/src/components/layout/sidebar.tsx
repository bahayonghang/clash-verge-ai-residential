import { useState, type CSSProperties, type KeyboardEvent, type PointerEvent } from "react";
import {
  Bell,
  ChartColumn,
  Cpu,
  Globe,
  House,
  Info,
  LayoutDashboard,
  Link2,
  Radio,
  Route as RouteIcon,
  Settings,
  X,
  type LucideIcon
} from "lucide-react";
import type { RouteId } from "../../dto";
import { t, type UiLocale } from "../../i18n";
import { cn, formatTemplate } from "../../lib/utils";
import { BRAND_MARK, BUSINESS_ROUTES, isRouteId } from "../../nav-icons";
import { BUSINESS_NAV_TINTS } from "../../nav-tints";
import { SHELL_WIDTH_MAX, SHELL_WIDTH_MIN } from "../../shell-width";
import { StatusDot } from "../common/status-dot";
import { Button } from "../ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";

const ROUTE_LUCIDE: Record<RouteId, LucideIcon> = {
  overview: LayoutDashboard,
  live: Radio,
  residential: House,
  host: Globe,
  rule: RouteIcon,
  chain: Link2,
  process: Cpu,
  reports: ChartColumn,
  alerts: Bell,
  "settings-data": Settings
};

export function ShellResizeHandle({
  width,
  label,
  valueText,
  onPointerDown,
  onPointerMove,
  onPointerUp,
  onPointerCancel,
  onKeyDown,
  onKeyUp,
  onBlur
}: {
  width: number;
  label: string;
  valueText: string;
  onPointerDown: (event: PointerEvent<HTMLElement>) => void;
  onPointerMove: (event: PointerEvent<HTMLElement>) => void;
  onPointerUp: (event: PointerEvent<HTMLElement>) => void;
  onPointerCancel: (event: PointerEvent<HTMLElement>) => void;
  onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
  onKeyUp: (event: KeyboardEvent<HTMLElement>) => void;
  onBlur: () => void;
}) {
  return (
    <span
      id="shell-resize"
      data-shell-resize="1"
      className="shell-resize"
      role="separator"
      tabIndex={0}
      aria-orientation="vertical"
      aria-label={label}
      aria-valuemin={SHELL_WIDTH_MIN}
      aria-valuemax={SHELL_WIDTH_MAX}
      aria-valuenow={width}
      aria-valuetext={valueText}
      aria-keyshortcuts="ArrowLeft ArrowRight Home End"
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerCancel}
      onLostPointerCapture={onPointerCancel}
      onKeyDown={onKeyDown}
      onKeyUp={onKeyUp}
      onBlur={onBlur}
    />
  );
}

function BrandTitle({ locale }: { locale: UiLocale }) {
  if (locale === "en") {
    return (
      <h1 data-brand="en-stack" className="min-w-0 text-[1.05rem] leading-tight font-bold">
        <span className="block">Residential</span>
        <span className="block">Traffic Monitor</span>
      </h1>
    );
  }
  return (
    <h1 className="min-w-0 text-[1.05rem] leading-tight font-bold">
      {t(locale, "product.display_name")}
    </h1>
  );
}

export function Sidebar({
  locale,
  route,
  recovery,
  healthSession,
  healthLabel,
  width,
  onRouteChange,
  resize
}: {
  locale: UiLocale;
  route: RouteId;
  recovery: boolean;
  healthSession: string;
  healthLabel: string;
  width: number;
  onRouteChange: (route: RouteId) => void;
  resize: {
    onPointerDown: (event: PointerEvent<HTMLElement>) => void;
    onPointerMove: (event: PointerEvent<HTMLElement>) => void;
    onPointerUp: (event: PointerEvent<HTMLElement>) => void;
    onPointerCancel: (event: PointerEvent<HTMLElement>) => void;
    onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
    onKeyUp: (event: KeyboardEvent<HTMLElement>) => void;
    onBlur: () => void;
  };
}) {
  const [aboutOpen, setAboutOpen] = useState(false);

  return (
    <aside
      className="relative flex h-full shrink-0 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground"
      style={{ width: "var(--shell-width)", flexBasis: "var(--shell-width)" }}
      onClick={(event) => {
        const target = (event.target as HTMLElement).closest("[data-route]");
        if (!(target instanceof HTMLElement)) {
          return;
        }
        const id = target.getAttribute("data-route");
        if (id && isRouteId(id)) {
          onRouteChange(id);
        }
      }}
    >
      <div className="flex items-start gap-3 border-b border-sidebar-border p-4">
        <img
          src={BRAND_MARK}
          alt=""
          width={56}
          height={56}
          className="size-14 shrink-0 rounded-xl object-cover"
        />
        <div className="min-w-0 flex-1">
          <div className="flex items-start gap-2">
            <BrandTitle locale={locale} />
            <span className="mt-1 shrink-0">
              <StatusDot session={healthSession} label={healthLabel} />
            </span>
          </div>
          <p className="mt-1 line-clamp-3 break-words text-[11px] leading-4 text-muted-foreground/80">
            {t(locale, "product.slogan_sidebar")}
          </p>
        </div>
      </div>

      {recovery ? (
        <p className="px-4 py-3 text-sm text-amber-500">{t(locale, "shell.recovery")}</p>
      ) : (
        <nav
          className="flex flex-1 flex-col gap-[length:var(--nav-item-gap)] overflow-auto p-4"
          aria-label={t(locale, "nav.aria")}
        >
          {BUSINESS_ROUTES.map((id) => {
            const Icon = ROUTE_LUCIDE[id];
            const current = id === route;
            return (
              <button
                key={id}
                type="button"
                data-route={id}
                data-nav-tint={id}
                aria-current={current ? "page" : undefined}
                style={{ "--nav-tint": BUSINESS_NAV_TINTS[id] } as CSSProperties}
                className={cn(
                  "shell-nav-item flex w-full items-center gap-3 rounded-xl px-3 text-sm font-medium transition-colors",
                  current
                    ? "bg-primary text-primary-foreground shadow-sm"
                    : "text-muted-foreground hover:bg-secondary hover:text-foreground"
                )}
              >
                <span className="shell-nav-well" aria-hidden="true">
                  <Icon className="size-4" />
                </span>
                <span className="min-w-0 truncate">{t(locale, `route.${id}`)}</span>
              </button>
            );
          })}
        </nav>
      )}

      <div className="mt-auto space-y-1 border-t border-sidebar-border p-4" aria-label={t(locale, "nav.bottom")}>
        <button
          type="button"
          className="shell-nav-item flex w-full items-center gap-3 rounded-xl px-3 text-sm font-medium text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          onClick={() => setAboutOpen(true)}
        >
          <Info className="h-5 w-5 shrink-0" />
          <span className="min-w-0 truncate">{t(locale, "nav.about")}</span>
        </button>
        <button
          type="button"
          data-route="settings-data"
          aria-current={route === "settings-data" ? "page" : undefined}
          className={cn(
            "shell-nav-item flex w-full items-center gap-3 rounded-xl px-3 text-sm font-medium transition-colors",
            route === "settings-data"
              ? "bg-primary text-primary-foreground shadow-sm"
              : "text-muted-foreground hover:bg-secondary hover:text-foreground"
          )}
        >
          <Settings className="h-5 w-5 shrink-0" />
          <span className="min-w-0 truncate">{t(locale, "route.settings-data")}</span>
        </button>
      </div>

      <ShellResizeHandle
        width={width}
        label={t(locale, "shell.resize")}
        valueText={formatTemplate(t(locale, "shell.resize_value"), { width })}
        {...resize}
      />

      {aboutOpen ? (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 p-4"
          onClick={(event) => {
            if (event.target === event.currentTarget) {
              setAboutOpen(false);
            }
          }}
        >
          <Card className="w-full max-w-md">
            <CardHeader className="flex flex-row items-start justify-between gap-4">
              <CardTitle>{t(locale, "nav.about")}</CardTitle>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                aria-label={t(locale, "a11y.close")}
                onClick={() => setAboutOpen(false)}
              >
                <X className="h-4 w-4" />
              </Button>
            </CardHeader>
            <CardContent className="space-y-2 text-sm text-muted-foreground">
              <p className="text-foreground">{t(locale, "product.display_name")}</p>
              <p>{t(locale, "product.slogan")}</p>
              <p>{t(locale, "settings.about_license_value")}</p>
              <p>{t(locale, "settings.about_platform_value")}</p>
              <p>{t(locale, "settings.about_privacy_value")}</p>
            </CardContent>
          </Card>
        </div>
      ) : null}
    </aside>
  );
}
