fn main() {
    // rfd（tauri-plugin-dialog）静态导入 TaskDialogIndirect，需要 comctl32 v6
    // 的 SxS 清单。tauri-build 默认把清单编进 resource.lib 并以
    // rustc-link-arg-bins 只链接 bin 目标；cargo test 产生的可执行文件
    // （lib 单测、bin 单测、tests/ 集成测试）都拿不到该参数，加载时报
    // STATUS_ENTRYPOINT_NOT_FOUND。因此这里去掉 resource.lib 里的清单，
    // 改用链接器参数把同一份清单嵌入所有目标。
    let manifest = r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>
"#;
    let attributes = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    tauri_build::try_build(attributes).expect("tauri build");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
        let manifest_path = out.join("common-controls.manifest");
        std::fs::write(&manifest_path, manifest).expect("write manifest");
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
            manifest_path.display()
        );
    }
}
