import { describe, expect, it } from "vitest";
import packageJson from "../../../../package.json?raw";
import capability from "../../../../src-tauri/capabilities/default.json?raw";
import cargo from "../../../../src-tauri/Cargo.toml?raw";
import desktop from "../../../../src-tauri/src/c2/desktop.rs?raw";
import facade from "../../../../src-tauri/src/c2/facade.rs?raw";
import rustRoot from "../../../../src-tauri/src/lib.rs?raw";
import dto from "../../../dto.ts?raw";

describe("autostart production boundary", () => {
  it("uses the official Rust plugin with the canonical background argument", () => {
    expect(cargo).toContain('tauri-plugin-autostart = "2"');
    expect(rustRoot).toContain("tauri_plugin_autostart::init(");
    expect(rustRoot).toContain("Some(vec![crate::identity::AUTOSTART_ARGUMENT])");
    expect(rustRoot).toContain("get_autostart_state");
    expect(rustRoot).toContain("set_autostart_enabled");
  });

  it("keeps fake ownership and guest permissions out of production", () => {
    expect(facade).not.toContain("FakeAutostart");
    expect(desktop).toMatch(/#\[cfg\(test\)\][\s\S]*pub struct FakeAutostart/);
    expect(packageJson).not.toContain("@tauri-apps/plugin-autostart");
    expect(capability).not.toContain("autostart:");
  });

  it("does not duplicate OS truth into bootstrap preferences", () => {
    const bootstrap = dto.slice(dto.indexOf("export interface BootstrapDto"));
    expect(bootstrap.slice(0, bootstrap.indexOf("export interface OperationProgress"))).not.toMatch(
      /autostart/i
    );
  });
});
