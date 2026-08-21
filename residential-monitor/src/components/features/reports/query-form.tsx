import type { ReportQuery } from "../../../dto";
import type { ReportForm, ReportPreset } from "../../../format/report-view";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";
import { fieldClass, labelClass } from "../form-styles";

const PRESETS: ReportPreset[] = ["hour", "day", "7", "30", "month"];
const GRAINS: ReportQuery["granularity"][] = [
  "minute1",
  "minute2",
  "minute5",
  "minute10",
  "hour",
  "day",
  "month"
];
const GROUPS: ReportQuery["grouping"][] = ["host", "process", "rule", "chain", "network", "category"];
const TOPN = [10, 20, 50];

export function QueryForm({
  locale,
  form,
  topN,
  compare,
  loading,
  onForm,
  onTopN,
  onCompare,
  onRun
}: {
  locale: UiLocale;
  form: ReportForm;
  topN: number;
  compare: boolean;
  loading: boolean;
  onForm: (form: ReportForm) => void;
  onTopN: (value: number) => void;
  onCompare: (value: boolean) => void;
  onRun: () => void;
}) {
  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm text-muted-foreground">{t(locale, "report.same_token")}</p>
      <div className="flex flex-wrap items-end gap-3">
        <label className={labelClass}>
          {t(locale, "report.preset")}
          <select
            className={fieldClass}
            value={form.preset}
            onChange={(event) =>
              onForm({ ...form, preset: event.target.value as ReportPreset, windowSource: "preset" })
            }
          >
            {PRESETS.map((item) => (
              <option key={item} value={item}>
                {t(locale, `report.preset.${item}`)}
              </option>
            ))}
          </select>
        </label>
        <label className={labelClass}>
          {t(locale, "report.granularity")}
          <select
            className={fieldClass}
            value={form.granularity}
            onChange={(event) =>
              onForm({ ...form, granularity: event.target.value as ReportQuery["granularity"] })
            }
          >
            {GRAINS.map((item) => (
              <option key={item} value={item}>
                {t(locale, `report.granularity.${item}`)}
              </option>
            ))}
          </select>
        </label>
        <label className={labelClass}>
          {t(locale, "report.grouping")}
          <select
            className={fieldClass}
            value={form.grouping}
            onChange={(event) =>
              onForm({ ...form, grouping: event.target.value as ReportQuery["grouping"] })
            }
          >
            {GROUPS.map((item) => (
              <option key={item} value={item}>
                {t(locale, `report.grouping.${item}`)}
              </option>
            ))}
          </select>
        </label>
        <label className={labelClass}>
          {t(locale, "report.topn")}
          <select className={fieldClass} value={topN} onChange={(event) => onTopN(Number(event.target.value))}>
            {TOPN.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
        </label>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={compare}
            onChange={(event) => onCompare(event.target.checked)}
          />
          {t(locale, "report.comparison")}
        </label>
        <Button type="button" disabled={loading} onClick={onRun}>
          {t(locale, "report.run")}
        </Button>
      </div>
      {form.windowSource === "archive" ? (
        <p className="text-sm text-muted-foreground">{t(locale, "report.preset.archive")}</p>
      ) : null}
    </div>
  );
}
