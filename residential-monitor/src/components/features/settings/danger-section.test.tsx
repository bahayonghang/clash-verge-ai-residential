import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { DeleteReport } from "../../../dto";
import { DangerSection } from "./danger-section";

const partial: DeleteReport = {
  schemaVersion: 1,
  allDeclaredOk: false,
  items: [
    { id: "db", ok: true, existed: true, messageZh: "对象已处理" },
    { id: "logs", ok: false, existed: true, messageZh: "删除失败" }
  ],
  summaryZh: "部分对象未删除"
};

describe("DangerSection", () => {
  it("部分失败只渲染 summaryZh，不含硬编码成功文案", () => {
    const html = renderToStaticMarkup(
      <DangerSection
        locale="zh"
        preview={null}
        report={partial}
        onPreview={() => undefined}
        onConfirm={() => undefined}
      />
    );
    expect(html).toContain("部分对象未删除");
    expect(html).toContain("对象已处理");
    expect(html).not.toContain("已全部删除");
  });
});
