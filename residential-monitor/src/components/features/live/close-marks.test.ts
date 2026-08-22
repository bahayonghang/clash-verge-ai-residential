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
});
