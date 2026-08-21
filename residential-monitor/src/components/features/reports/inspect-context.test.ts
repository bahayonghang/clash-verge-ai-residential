import { describe, expect, it } from "vitest";
import { reportShareModel } from "../../../format/report-view";
import { retainInspectKey } from "./inspect-context";

const copy = { unknown: "未知", remainder: "其余" };

describe("retainInspectKey", () => {
  const share = reportShareModel(
    {
      totals: { download: 100 },
      rankings: [{ identity: "a", label: "a.example", upload: 1, download: 60 }],
      drilldownCapability: { exactTopN: true }
    },
    copy
  );
  const series = [{ bucketUtc: 8, upload: 4, download: 9 }];

  it("新 ReportResult 不再包含的 pinned / hover key 被清空", () => {
    expect(retainInspectKey("rank:a", share, series)).toBe("rank:a");
    expect(retainInspectKey("trend:8", share, series)).toBe("trend:8");
    expect(retainInspectKey("rank:missing", share, series)).toBeNull();
    expect(retainInspectKey("trend:1", share, series)).toBeNull();
  });
});
