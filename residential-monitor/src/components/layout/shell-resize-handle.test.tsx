import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { ShellResizeHandle } from "./sidebar";

const noop = (): void => undefined;

describe("ShellResizeHandle", () => {
  it("保留分隔条 ARIA slider 语义", () => {
    const html = renderToStaticMarkup(
      <ShellResizeHandle
        width={220}
        label="调整侧栏宽度"
        valueText="220 像素"
        onPointerDown={noop}
        onPointerMove={noop}
        onPointerUp={noop}
        onPointerCancel={noop}
        onKeyDown={noop}
        onKeyUp={noop}
        onBlur={noop}
      />
    );
    expect(html).toContain('id="shell-resize"');
    expect(html).toContain('role="separator"');
    expect(html).toContain('aria-orientation="vertical"');
    expect(html).toContain('aria-valuemin="160"');
    expect(html).toContain('aria-valuemax="352"');
    expect(html).toContain('aria-valuenow="220"');
    expect(html).toContain("aria-valuetext");
    expect(html).toContain("ArrowLeft ArrowRight Home End");
    expect(html).toContain("tabindex=\"0\"");
  });
});
