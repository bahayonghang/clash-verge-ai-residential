import { describe, expect, it } from "vitest";
import source from "./use-settings.ts?raw";
import connectionSource from "../components/features/settings/connection-section.tsx?raw";

describe("secret 与保存", () => {
  it("保存默认写入本机凭据", () => {
    expect(source).toMatch(/sessionOnly:\s*false/);
    expect(source).not.toMatch(/sessionOnly:\s*true/);
  });

  it("设置 hook 不把错误打进 console", () => {
    expect(source).not.toMatch(/console\.(log|error|debug|info|warn)/);
  });

  it("连接分区密码框不把密钥写进 data-* 或 title", () => {
    expect(connectionSource).toContain('id="controller-secret"');
    expect(connectionSource).toContain('type={visible ? "text" : "password"}');
    expect(connectionSource).not.toMatch(/data-[a-zA-Z-]+=\{secret\}/);
    expect(connectionSource).not.toMatch(/title=\{secret\}/);
    expect(connectionSource).not.toMatch(/console\./);
  });
});
