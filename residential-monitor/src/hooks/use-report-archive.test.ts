import { describe, expect, it } from "vitest";
import { pickLatestArchive } from "./use-report-archive";

describe("pickLatestArchive", () => {
  it("优先日档案，否则小时档案", () => {
    expect(
      pickLatestArchive({
        schemaVersion: 1,
        items: [
          {
            archiveId: "h1",
            kind: "hour",
            rangeStartUtc: 1,
            rangeEndUtc: 2,
            displayTimezone: "local",
            grouping: "host",
            status: "ok",
            generatedUtc: 1,
            dataVersion: null,
            coverageStatus: null,
            totalsUpload: null,
            totalsDownload: 1,
            connectionCount: null,
            errorCode: null,
            noteZh: null
          },
          {
            archiveId: "d1",
            kind: "day",
            rangeStartUtc: 1,
            rangeEndUtc: 2,
            displayTimezone: "local",
            grouping: "host",
            status: "ok",
            generatedUtc: 1,
            dataVersion: null,
            coverageStatus: null,
            totalsUpload: null,
            totalsDownload: 2,
            connectionCount: null,
            errorCode: null,
            noteZh: null
          }
        ],
        next: null
      })?.archiveId
    ).toBe("d1");
  });
});
