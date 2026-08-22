//! 生产文件对话框适配器。测试用 `c2::shell::FakeFileDialog`。

use crate::c2::shell::{dialog_spec, FileDialogPort, FileMode, FilePurpose};
use crate::i18n::UiLocale;
use std::path::PathBuf;
use tauri_plugin_dialog::DialogExt;

pub struct TauriFileDialog {
    pub app: tauri::AppHandle,
}

impl FileDialogPort for TauriFileDialog {
    fn pick(&self, locale: UiLocale, purpose: FilePurpose, mode: FileMode) -> Option<PathBuf> {
        let spec = dialog_spec(locale, purpose);
        let extensions: Vec<&str> = spec.extensions.iter().map(String::as_str).collect();
        let builder = self
            .app
            .dialog()
            .file()
            .set_title(spec.title)
            .set_file_name(spec.file_name)
            .add_filter(spec.filter_name, &extensions);
        let picked = match mode {
            FileMode::Open => builder.blocking_pick_file(),
            FileMode::Save => builder.blocking_save_file(),
        };
        picked.and_then(|path| path.into_path().ok())
    }
}
