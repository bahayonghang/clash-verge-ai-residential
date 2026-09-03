import { describe, expect, it } from "vitest";
import source from "./use-settings.ts?raw";
import connectionSource from "../components/features/settings/connection-section.tsx?raw";
import settingsPageSource from "../components/features/settings/index.tsx?raw";

describe("secret 与保存", () => {
  it("未保存配置使用可提交的地址和重点目标默认值", () => {
    expect(source).toContain('const DEFAULT_CONTROLLER_ADDRESS = "127.0.0.1:9097";');
    expect(source).toContain('const DEFAULT_TARGETS = "家宽";');
    expect(source).toMatch(
      /useState\(boot\?\.settings\.address \|\| DEFAULT_CONTROLLER_ADDRESS\)/
    );
    expect(source).toMatch(
      /setAddress\(boot\.settings\.address \|\| DEFAULT_CONTROLLER_ADDRESS\)/
    );
    expect(source).toContain("useState(DEFAULT_TARGETS)");
    expect(connectionSource).not.toContain('placeholder="127.0.0.1:9097"');
  });

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

  it("自启动使用独立请求控制器并在进入连接分区时刷新", () => {
    expect(source).toContain("AutostartRequestController");
    expect(source).toContain("loadAutostart");
    expect(source).toContain("setAutostartEnabled");
    expect(settingsPageSource).toContain("void loadAutostart()");
    expect(connectionSource).toContain("<StartupSection");
  });
});
