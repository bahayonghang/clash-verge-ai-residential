import { describe, expect, it } from "vitest";
import { promoteAcceptedToClosed, setCloseMark } from "./close-marks";

describe("closeMarks", () => {
  it("按 CloseState 三态写入并在 remove 后把 accepted 标为 closed", () => {
    let marks = new Map<string, "accepted" | "closed" | "unconfirmed">();
    marks = setCloseMark(marks, "0:a", "accepted");
    marks = setCloseMark(marks, "0:b", "unconfirmed");
    marks = setCloseMark(marks, "0:c", "closed");
    expect(marks.get("0:a")).toBe("accepted");
    const next = promoteAcceptedToClosed(marks, ["0:a", "0:b"]);
    expect(next.get("0:a")).toBe("closed");
    expect(next.get("0:b")).toBe("unconfirmed");
    expect(next.get("0:c")).toBe("closed");
  });

  it("查询页或 Channel 身份集里消失的 accepted 行变为 closed", () => {
    const prev = new Set(["0:a", "0:b"]);
    const current = new Set(["0:b"]);
    const disappeared = [...prev].filter((id) => !current.has(id));
    const marks = promoteAcceptedToClosed(new Map([["0:a", "accepted" as const], ["0:b", "accepted" as const]]), disappeared);
    expect(marks.get("0:a")).toBe("closed");
    expect(marks.get("0:b")).toBe("accepted");
  });
});
