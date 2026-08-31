import { describe, expect, it } from "vitest";
import { RESIDENTIAL_ACCOUNTING_FILTER } from "../../../format/rank";
import { buildResidentialManualQuery, isResidentialManualReport } from "./report-section";

describe("家宽手动报告查询", () => {
  it("按家宽子集内的 Host 生成并保留上下行字段", () => {
    const query = buildResidentialManualQuery(
      { preset: "24h", startUtc: 0, endUtc: 86_400_000 },
      "historical"
    );
    expect(query.grouping).toBe("host");
    expect(query.filters).toEqual({
      category: RESIDENTIAL_ACCOUNTING_FILTER,
      host: null,
      process: null,
      rule: null,
      chain: null,
      network: null
    });
    expect(query.sort).toEqual({ field: "download", descending: true });
    expect(query.targetPolicy).toBe("historical");
  });

  it("只切换 targetPolicy，不改变地址拆解口径", () => {
    const query = buildResidentialManualQuery(
      { preset: "1h", startUtc: 0, endUtc: 3_600_000 },
      "current"
    );
    expect(query.targetPolicy).toBe("current");
    expect(query.grouping).toBe("host");
    expect(query.filters.category).toBe(RESIDENTIAL_ACCOUNTING_FILTER);
  });

  it("只把 host + 家宽 filter 认作家宽手动报告", () => {
    const query = buildResidentialManualQuery(
      { preset: "24h", startUtc: 0, endUtc: 86_400_000 },
      "historical"
    );
    expect(isResidentialManualReport({ queryEcho: query })).toBe(true);
    expect(
      isResidentialManualReport({
        queryEcho: { ...query, grouping: "rule" }
      })
    ).toBe(false);
    expect(
      isResidentialManualReport({
        queryEcho: { ...query, filters: { ...query.filters, category: null } }
      })
    ).toBe(false);
  });
});
