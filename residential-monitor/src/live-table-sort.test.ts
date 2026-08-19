import { describe, expect, it } from "vitest";
import { nextLiveSort, sortAria } from "./live-table-sort";

describe("live table header sort cycle", () => {
  it("starts descending on a new column then returns to identity", () => {
    const first = nextLiveSort("download", { sortField: "identity", descending: false });
    expect(first).toEqual({ sortField: "download", descending: true });
    const second = nextLiveSort("download", first);
    expect(second).toEqual({ sortField: "download", descending: false });
    const third = nextLiveSort("download", second);
    expect(third).toEqual({ sortField: "identity", descending: false });
  });

  it("switching column restarts at descending", () => {
    const next = nextLiveSort("upload", { sortField: "download", descending: true });
    expect(next).toEqual({ sortField: "upload", descending: true });
    expect(sortAria("upload", next)).toBe("descending");
    expect(sortAria("download", next)).toBe("none");
  });
});
