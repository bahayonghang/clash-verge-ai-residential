import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

export function formatTemplate(template: string, vars: Record<string, string | number>): string {
  let text = template;
  for (const [name, value] of Object.entries(vars)) {
    text = text.replaceAll(`{${name}}`, String(value));
  }
  return text;
}

export function invokeErrorZh(error: unknown, fallback: string): string {
  if (!error || typeof error !== "object") {
    return fallback;
  }
  const rec = error as Record<string, unknown>;
  if (typeof rec.messageZh === "string" && rec.messageZh.length > 0) {
    return rec.messageZh;
  }
  if (typeof rec.message === "string") {
    try {
      const parsed = JSON.parse(rec.message) as Record<string, unknown>;
      if (typeof parsed.messageZh === "string" && parsed.messageZh.length > 0) {
        return parsed.messageZh;
      }
    } catch {
      /* 非 JSON */
    }
  }
  return fallback;
}
