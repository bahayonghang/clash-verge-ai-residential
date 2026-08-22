import type { ReportResult } from "../../../dto";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { formatTemplate } from "../../../lib/utils";

export function AttributionQualityNote({
  locale,
  result
}: {
  locale: UiLocale;
  result: ReportResult | null;
}) {
  if (!result) {
    return null;
  }
  const quality = result.attributionQuality;
  return (
    <p
      data-attribution-status={quality.status}
      className="text-xs leading-relaxed text-muted-foreground"
    >
      {formatTemplate(t(locale, "dimension.attribution_quality"), {
        status: t(locale, `dimension.attribution.${quality.status}`),
        known: formatBytes(quality.knownUpload + quality.knownDownload, "—"),
        missing: formatBytes(quality.missingUpload + quality.missingDownload, "—"),
        knownConnections: quality.knownConnections,
        totalConnections: quality.knownConnections + quality.missingConnections
      })}
    </p>
  );
}
