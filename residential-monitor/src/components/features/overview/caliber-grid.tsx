import { Activity, AlertTriangle, ArrowUpRight, Gauge, Layers, Unplug } from "lucide-react";
import type { LiveOverview } from "../../../dto";
import { formatUtc } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { healthOf } from "../../../lib/health";
import { formatTemplate } from "../../../lib/utils";
import { CaliberCard, SessionCaliberCard } from "./caliber-card";

export function CaliberGrid({ locale, overview }: { locale: UiLocale; overview: LiveOverview }) {
  const unknown = t(locale, "common.unknown");
  const health = healthOf(locale, overview.health.session);
  const coverage = overview.observationPhase !== "current"
    ? t(locale, `overview.phase.${overview.observationPhase}`)
    : overview.coverageKind
    ? formatTemplate(t(locale, "overview.coverage_gap"), {
        kind: overview.coverageKind,
        reason: overview.coverageReason && overview.coverageReason.length > 0 ? overview.coverageReason : unknown
      })
    : formatTemplate(t(locale, "overview.coverage_ok"), {
        time: formatUtc(overview.lastSampleUtc, t(locale, "common.no_sample"))
      });
  return (
    <section
      className="grid grid-cols-2 gap-2.5 sm:grid-cols-3 lg:grid-cols-6"
      aria-label={t(locale, "overview.aria")}
    >
      <CaliberCard
        locale={locale}
        icon={<Gauge />}
        label={t(locale, "overview.meter")}
        color="#3B82F6"
        upload={overview.meterUpload}
        download={overview.meterDownload}
        phase={overview.observationPhase}
        uploadField="meter-upload"
        downloadField="meter-download"
      />
      <CaliberCard
        locale={locale}
        icon={<Layers />}
        label={t(locale, "overview.attr")}
        color="#8B5CF6"
        upload={overview.attributedUpload}
        download={overview.attributedDownload}
        phase={overview.observationPhase}
        uploadField="attr-upload"
        downloadField="attr-download"
      />
      <CaliberCard
        locale={locale}
        icon={<Unplug />}
        label={t(locale, "overview.other")}
        color="#06B6D4"
        upload={overview.otherUpload}
        download={overview.otherDownload}
        phase={overview.observationPhase}
        uploadField="other-upload"
        downloadField="other-download"
      />
      <CaliberCard
        locale={locale}
        icon={<AlertTriangle />}
        label={t(locale, "overview.gap")}
        color="#F59E0B"
        upload={overview.gapUpload}
        download={overview.gapDownload}
        phase={overview.observationPhase}
        uploadField="gap-upload"
        downloadField="gap-download"
      />
      <CaliberCard
        locale={locale}
        icon={<ArrowUpRight />}
        label={t(locale, "overview.over")}
        color="#EF4444"
        upload={overview.overUpload}
        download={overview.overDownload}
        phase={overview.observationPhase}
        uploadField="over-upload"
        downloadField="over-download"
      />
      <SessionCaliberCard
        locale={locale}
        icon={<Activity />}
        color="#10B981"
        activeCount={overview.activeCount}
        phase={overview.observationPhase}
        coverage={coverage}
        healthTitle={health.title}
        healthAction={health.action}
      />
    </section>
  );
}
