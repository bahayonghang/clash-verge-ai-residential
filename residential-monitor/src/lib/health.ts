import { healthAction, healthTitle, t, type UiLocale } from "../i18n";

export type HealthTone = "ok" | "warn" | "bad";

export function healthOf(locale: UiLocale, session: string): { title: string; action: string } {
  const title = healthTitle(locale, session);
  if (title === `health.${session}`) {
    return { title: session, action: t(locale, "health.view_diag") };
  }
  return { title, action: healthAction(locale, session) };
}

export function healthTone(session: string): HealthTone {
  if (session === "connected") {
    return "ok";
  }
  if (session === "connecting" || session === "no_data" || session === "paused") {
    return "warn";
  }
  return "bad";
}
