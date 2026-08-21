import { describe, expect, it } from "vitest";
import {
  applyReportScrollReset,
  inspectGroup,
  inspectKeysMatch,
  rankingInspectKey,
  reportInspectModel,
  trendInspectKey
} from "./report-inspect";
import { reportShareModel } from "./report-view";

const copy = { unknown: "未知", remainder: "其余" };

describe("inspect keys", () => {
  it("names ranking and remainder rows", () => {
    expect(rankingInspectKey({ kind: "rank", identity: "a.example" })).toBe("rank:a.example");
    expect(rankingInspectKey({ kind: "remainder", identity: null })).toBe("remainder");
  });

  it("groups trend up/down with the bucket row", () => {
    expect(trendInspectKey(12)).toBe("trend:12");
    expect(trendInspectKey(12, "up")).toBe("trend:12:up");
    expect(inspectGroup("trend:12:down")).toBe("trend:12");
    expect(inspectKeysMatch("trend:12:up", "trend:12")).toBe(true);
    expect(inspectKeysMatch("rank:a", "remainder")).toBe(false);
  });
});

describe("reportInspectModel", () => {
  const share = reportShareModel(
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
  const series = [{ bucketUtc: 8, upload: 4, download: 9 }];

  it("returns pie fields for a ranking key", () => {
    expect(reportInspectModel("rank:a", share, series)).toMatchObject({
      surface: "pie",
      label: "a.example",
      download: 60,
      share: 0.6
    });
  });

  it("returns remainder and trend payloads", () => {
    expect(reportInspectModel("remainder", share, series)).toMatchObject({
      surface: "pie",
      label: "其余",
      upload: null,
      download: 30
    });
    expect(reportInspectModel("trend:8:down", share, series)).toEqual({
      surface: "trend",
      key: "trend:8:down",
      bucketUtc: 8,
      direction: "down",
      upload: 4,
      download: 9
    });
  });

  it("returns null when the key is missing from the current result", () => {
    expect(reportInspectModel("rank:missing", share, series)).toBeNull();
    expect(reportInspectModel("trend:1", share, series)).toBeNull();
  });
});

describe("applyReportScrollReset", () => {
  it("zeros captured scroll when the snapshot token changed", () => {
    const captured = { workspace: 40, wraps: { topn: 80, trend: 12 } };
    expect(applyReportScrollReset(captured, true)).toEqual({ workspace: 0, wraps: {} });
    expect(applyReportScrollReset(captured, false)).toEqual(captured);
  });
});
