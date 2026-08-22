import { describe, expect, it } from "vitest";
import { defaultFilterUnit, toQueryClause, toQueryMagnitude, unitsForField } from "./live-filter-units";

describe("live filter unit conversion", () => {
  it("converts KiB and minutes to integers", () => {
    expect(toQueryMagnitude("38.6", "KiB")).toBe("39526");
    expect(toQueryMagnitude("3", "min")).toBe("180000");
  });

  it("rejects empty, negative, and overflow", () => {
    expect(toQueryMagnitude("", "KiB")).toBeNull();
    expect(toQueryMagnitude("-1", "B")).toBeNull();
    expect(toQueryMagnitude("1e30", "GiB")).toBeNull();
  });

  it("picks default units by field", () => {
    expect(defaultFilterUnit("download")).toBe("KiB");
    expect(defaultFilterUnit("duration")).toBe("min");
    expect(unitsForField("rateDownload")).toEqual(["B", "KiB", "MiB"]);
  });

  it("sends converted integers and drops the unit field", () => {
    expect(
      toQueryClause({ field: "download", mode: "gte", value: "1", unit: "KiB" })
    ).toEqual({ field: "download", mode: "gte", value: "1024" });
    expect(toQueryClause({ field: "host", mode: "contains", value: "grok.com" })).toEqual({
      field: "host",
      mode: "contains",
      value: "grok.com"
    });
    expect(toQueryClause({ field: "duration", mode: "lt", value: "", unit: "min" }).value).toBe("");
  });
});
