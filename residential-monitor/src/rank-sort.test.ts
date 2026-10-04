import { describe, expect, it } from "vitest";
import { DEFAULT_RANK_SORT, isRankSortField, nextRankSort, rankSortAria } from "./rank-sort";

describe("维度排名表头排序", () => {
  it("默认下行降序，切到上行仍降序", () => {
    expect(DEFAULT_RANK_SORT).toEqual({ field: "download", descending: true });
    expect(nextRankSort("upload", DEFAULT_RANK_SORT)).toEqual({ field: "upload", descending: true });
    expect(nextRankSort("name", DEFAULT_RANK_SORT)).toEqual({ field: "name", descending: false });
    expect(nextRankSort("identity", DEFAULT_RANK_SORT)).toEqual({
      field: "identity",
      descending: false
    });
  });

  it("同一列再点切换方向", () => {
    const upload = nextRankSort("upload", DEFAULT_RANK_SORT);
    expect(nextRankSort("upload", upload)).toEqual({ field: "upload", descending: false });
    expect(nextRankSort("download", DEFAULT_RANK_SORT)).toEqual({
      field: "download",
      descending: false
    });
  });

  it("连接数列不在白名单", () => {
    expect(isRankSortField("connections")).toBe(false);
    expect(isRankSortField("share")).toBe(false);
    expect(isRankSortField("upload")).toBe(true);
    expect(rankSortAria("download", DEFAULT_RANK_SORT)).toBe("descending");
    expect(rankSortAria("upload", DEFAULT_RANK_SORT)).toBe("none");
  });
});
