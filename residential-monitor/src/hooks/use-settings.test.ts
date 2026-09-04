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

  it("secret、连接保存、About、数据操作分序号，About 不丢弃保存结果", () => {
    expect(source).toContain("secretSeq");
    expect(source).toContain("connectionSeq");
    expect(source).toContain("aboutSeq");
    expect(source).toContain("dataSeq");
    expect(source).not.toMatch(/const seq = useRef\(0\)/);
    expect(source).toMatch(
      /const saveConnection = useCallback\(async \(\): Promise<void> => \{\s*const token = \+\+connectionSeq\.current/
    );
    expect(source).toMatch(/const token = \+\+aboutSeq\.current;\s*const fallback = t\(locale, "settings\.about_fail"\)/);
  });

  it("secret 读取失败设置 errorZh 与重试，不把空密码框当成无 secret", () => {
    expect(source).toContain("secret.load_fail");
    expect(source).toContain("setSecretErrorZh(message)");
    expect(source).toContain("setErrorZh(message)");
    expect(source).toContain("retrySecret");
    expect(source).not.toMatch(/secretLoaded\.current = true;\s*if \(!isTauriRuntime/);
    expect(connectionSource).toContain("secretErrorZh");
    expect(connectionSource).toContain("secret.retry");
    expect(settingsPageSource).toContain("retrySecret");
  });
});
