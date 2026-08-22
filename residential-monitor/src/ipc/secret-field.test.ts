import { describe, expect, it } from "vitest";
import { secretFieldMarkup } from "./secret-field";
import settingsSource from "../hooks/use-settings.ts?raw";

describe("secret-field", () => {
  it("默认是 password，并提供显示按钮", () => {
    const html = secretFieldMarkup();
    expect(html).toContain('id="controller-secret"');
    expect(html).toContain('type="password"');
    expect(html).toContain('id="toggle-secret"');
    expect(html).toContain('aria-label="显示密钥"');
    expect(html).not.toMatch(/\svalue="/);
  });

  it("markup 不含密钥明文", () => {
    expect(secretFieldMarkup()).not.toContain("echo-secret");
    expect(secretFieldMarkup()).not.toContain("${");
  });

  it("保存默认写入本机凭据，不是 session-only", () => {
    expect(settingsSource).toMatch(/sessionOnly:\s*false/);
    expect(settingsSource).not.toMatch(/sessionOnly:\s*true/);
  });
});
