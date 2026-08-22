import { describe, expect, it } from "vitest";
import type { ReportQuery } from "../dto";
import {
  applyPresetRange,
  defaultReportForm,
  formFromQueryEcho,
  formatSharePct,
  presetFromSpan,
  reportShareModel,
  reportTrendModel
} from "./report-view";

const copy = { unknown: "未知", remainder: "其余" };

function query(over: Partial<ReportQuery> = {}): ReportQuery {
  return {
    rangeStartUtc: 0,
    rangeEndUtc: 3600,
    displayTimezone: "local",
    granularity: "hour",
    filters: { category: null, host: null, process: null, rule: null, chain: null, network: null },
    grouping: "host",
    targetPolicy: "historical",
    comparison: { previousEqualWindow: true },
    sort: { field: "download", descending: true },
    page: { limit: 200, after: null },
    topN: 20,
    includeSessions: false,
    ...over
  };
}

describe("presetFromSpan / formFromQueryEcho", () => {
  it("maps exact hour and day spans", () => {
    expect(presetFromSpan(3600)).toBe("hour");
    expect(presetFromSpan(86400)).toBe("day");
    expect(presetFromSpan(7 * 86400)).toBe("7");
    expect(presetFromSpan(30 * 86400)).toBe("30");
    expect(presetFromSpan(82800)).toBeNull();
  });

  it("marks unmatched archive windows without claiming last hour", () => {
    const form = formFromQueryEcho(query({ rangeStartUtc: 0, rangeEndUtc: 90000, granularity: "day", grouping: "process" }));
    expect(form.windowSource).toBe("archive");
    expect(form.granularity).toBe("day");
    expect(form.grouping).toBe("process");
    expect(form.preset).toBe("hour");
  });

  it("maps a 86400 window to the day preset", () => {
    const form = formFromQueryEcho(query({ rangeStartUtc: 10, rangeEndUtc: 10 + 86400, grouping: "rule" }));
    expect(form.windowSource).toBe("preset");
    expect(form.preset).toBe("day");
    expect(form.grouping).toBe("rule");
  });
});

describe("applyPresetRange", () => {
  it("keeps the archive range while windowSource is archive", () => {
    const form = { ...defaultReportForm(), windowSource: "archive" as const, granularity: "day" as const };
    const next = applyPresetRange(query(), form, 10_000, { start: 1, end: 2, timezone: "local" });
    expect(next.rangeStartUtc).toBe(1);
    expect(next.rangeEndUtc).toBe(2);
    expect(next.granularity).toBe("day");
  });

  it("uses the rolling preset span after the user picks a preset", () => {
    const form = { ...defaultReportForm(), preset: "7" as const };
    const next = applyPresetRange(query(), form, 7 * 86400);
    expect(next.rangeStartUtc).toBe(0);
    expect(next.rangeEndUtc).toBe(7 * 86400);
  });
});

describe("reportShareModel", () => {
  it("adds a remainder row and draws a pie when the gap is positive", () => {
    const model = reportShareModel(
      {
        totals: { download: 100 },
        rankings: [
          { identity: "a", label: "a.example", upload: 1, download: 60 },
          { identity: "b", label: "", upload: 2, download: 10 }
        ],
        drilldownCapability: { exactTopN: true }
      },
      copy
    );
    expect(model.drawPie).toBe(true);
    expect(model.remainder).toBe(30);
    expect(model.rows).toHaveLength(3);
    expect(model.rows[1]?.label).toBe("未知");
    expect(model.rows[2]).toMatchObject({ kind: "remainder", label: "其余", upload: null, download: 30, share: 0.3 });
  });

  it("omits remainder and still draws when the gap is zero", () => {
    const model = reportShareModel(
      {
        totals: { download: 50 },
        rankings: [{ identity: "a", label: "a", upload: 1, download: 50 }],
        drilldownCapability: { exactTopN: true }
      },
      copy
    );
    expect(model.drawPie).toBe(true);
    expect(model.rows).toHaveLength(1);
  });

  it("does not draw a pie when rankings exceed totals", () => {
    const model = reportShareModel(
      {
        totals: { download: 10 },
        rankings: [{ identity: "a", label: "a", upload: 0, download: 12 }],
        drilldownCapability: { exactTopN: true }
      },
      copy
    );
    expect(model.drawPie).toBe(false);
    expect(model.rows).toHaveLength(1);
    expect(model.rows[0]?.share).toBe(1.2);
  });

  it("does not draw a pie when the denominator is 0", () => {
    const model = reportShareModel(
      {
        totals: { download: 0 },
        rankings: [{ identity: "a", label: "a", upload: 0, download: 0 }],
        drilldownCapability: { exactTopN: true }
      },
      copy
    );
    expect(model.drawPie).toBe(false);
    expect(model.rows[0]?.share).toBeNull();
  });

  it("hides rankings when exact Top N is unsupported", () => {
    const model = reportShareModel(
      {
        totals: { download: 100 },
        rankings: [{ identity: "a", label: "a", upload: 1, download: 40 }],
        drilldownCapability: { exactTopN: false }
      },
      copy
    );
    expect(model.capabilityUnsupported).toBe(true);
    expect(model.drawPie).toBe(false);
    expect(model.rows).toEqual([]);
  });
});

describe("formatSharePct", () => {
  it("formats percents and tiny remainders", () => {
    expect(formatSharePct(0.256, "未知")).toBe("25.6%");
    expect(formatSharePct(0.0004, "未知")).toBe("<0.1%");
    expect(formatSharePct(null, "未知")).toBe("未知");
  });
});

describe("reportTrendModel", () => {
  it("returns empty when series is empty", () => {
    expect(reportTrendModel([])).toEqual({ kind: "empty", max: 1, points: [] });
  });

  it("marks a single bucket without a line span", () => {
    const model = reportTrendModel([{ bucketUtc: 1, upload: 4, download: 8 }]);
    expect(model.kind).toBe("single");
    expect(model.max).toBe(8);
    expect(model.points[0]?.x).toBe(0.5);
    expect(model.points[0]?.yDown).toBe(1);
    expect(model.points[0]?.yUp).toBe(0.5);
  });

  it("spaces multiple buckets from 0 to 1", () => {
    const model = reportTrendModel([
      { bucketUtc: 1, upload: 0, download: 0 },
      { bucketUtc: 2, upload: 1, download: 2 },
      { bucketUtc: 3, upload: 2, download: 4 }
    ]);
    expect(model.kind).toBe("multi");
    expect(model.points.map((point) => point.x)).toEqual([0, 0.5, 1]);
    expect(model.max).toBe(4);
  });
});
