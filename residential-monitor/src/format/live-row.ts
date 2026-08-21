import type { LiveConnectionView } from "../dto";
import { formatBytes, formatRate } from "./units";
import { t, type UiLocale } from "../i18n";

export function joinHostPort(host: string | null | undefined, port: string | null | undefined): string | null {
  if (!host) {
    return null;
  }
  return port ? `${host}:${port}` : host;
}

export function formatChains(chains: string[]): string | null {
  return chains.length > 0 ? chains.join(" / ") : null;
}

export function formatRule(rule: string | null, payload: string | null): string | null {
  if (!rule) {
    return null;
  }
  return payload ? `${rule}(${payload})` : rule;
}

export function formatType(inbound: string | null | undefined, network: string | null | undefined): string | null {
  if (inbound && network) {
    return `${inbound}(${network})`;
  }
  return inbound || network || null;
}

export function formatRelative(durationMs: number | null, locale: UiLocale): string | null {
  if (durationMs == null) {
    return null;
  }
  const seconds = Math.max(0, Math.floor(durationMs / 1000));
  if (seconds < 10) {
    return t(locale, "live.rel.now");
  }
  if (seconds < 60) {
    return t(locale, "live.rel.seconds").replace("{n}", String(seconds));
  }
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return t(locale, "live.rel.minutes").replace("{n}", String(minutes));
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return t(locale, "live.rel.hours").replace("{n}", String(hours));
  }
  return t(locale, "live.rel.days").replace("{n}", String(Math.floor(hours / 24)));
}

export function displayLiveRow(row: LiveConnectionView, locale: UiLocale, unknown: string): {
  host: string;
  download: string;
  upload: string;
  dlSpeed: string;
  ulSpeed: string;
  chains: string;
  rule: string;
  process: string;
  time: string;
  source: string;
  destination: string;
  type: string;
} {
  return {
    host: joinHostPort(row.host ?? row.destinationIp, row.destinationPort) ?? unknown,
    download: formatBytes(row.download, unknown),
    upload: formatBytes(row.upload, unknown),
    dlSpeed: formatRate(row.rateDownload, unknown),
    ulSpeed: formatRate(row.rateUpload, unknown),
    chains: formatChains(row.chains) ?? unknown,
    rule: formatRule(row.rule, row.rulePayload) ?? unknown,
    process: row.processName && row.processName.length > 0 ? row.processName : unknown,
    time: formatRelative(row.durationMs, locale) ?? unknown,
    source: joinHostPort(row.sourceIp, row.sourcePort) ?? unknown,
    destination: joinHostPort(row.destinationIp, row.destinationPort) ?? unknown,
    type: formatType(row.inbound, row.network) ?? unknown
  };
}
