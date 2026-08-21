import type { KeyboardEvent, PointerEvent, ReactNode } from "react";
import type { RouteId } from "../../dto";
import type { UiLocale } from "../../i18n";
import { Sidebar } from "./sidebar";

export function Shell({
  locale,
  route,
  recovery,
  healthSession,
  healthLabel,
  width,
  onRouteChange,
  resize,
  header,
  children,
  errorZh
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
  header: ReactNode;
  children: ReactNode;
  errorZh: string | null;
}) {
  return (
    <div className="flex h-full min-h-0 overflow-hidden">
      <Sidebar
        locale={locale}
        route={route}
        recovery={recovery}
        healthSession={healthSession}
        healthLabel={healthLabel}
        width={width}
        onRouteChange={onRouteChange}
        resize={resize}
      />
      <div className="flex min-w-0 flex-1 flex-col">
        {header}
        <main id="workspace" className="min-h-0 flex-1 overflow-auto p-4" tabIndex={-1}>
          {children}
          {errorZh ? (
            <p className="mt-3 text-sm text-destructive" role="alert">
              {errorZh}
            </p>
          ) : null}
        </main>
      </div>
    </div>
  );
}
