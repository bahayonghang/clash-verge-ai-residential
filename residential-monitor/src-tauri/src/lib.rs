pub mod accounting;
pub mod bench;
pub mod c0_contract;
pub mod c2;
pub mod c3;
pub mod candidate_schema;
pub mod controller;
pub mod credential;
pub mod evidence;
pub mod identity;
pub mod live;
pub mod session;
pub mod sqlite_probe;
pub mod storage;
pub mod transport;
pub mod workload;

use crate::session::ControllerSession;
use c2::desktop::InstanceClaim;
#[cfg(not(windows))]
use c2::desktop::ProcessSingleInstance;
use c2::facade::{parse_socket, AppErrorDto, AppFacade, BootstrapDto};
use c2::hub::{LiveConnectionView, MonitorStreamMessage};
use c2::query::{ConnectionPage, ConnectionQuery};
use c2::settings::ControllerSettings;
use c2::shell::{FileMode, FilePurpose, OperationProgress, RecoveryStatus, RouteDescriptor};
use c3::export::{ExportPreview, ExportSpec};
use c3::query::{ReportQuery, ReportResult};
use c3::retention::RetentionPreview;
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::{Emitter, Manager, State};

#[cfg(not(windows))]
fn claim_process_instance() -> InstanceClaim {
    ProcessSingleInstance::claim_first().claim()
}

#[cfg(windows)]
fn try_windows_single_instance() -> InstanceClaim {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;

    let name = format!("{}.single-instance", crate::identity::IDENTIFIER);
    let wide: Vec<u16> = std::ffi::OsString::from(&name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let handle = unsafe { CreateMutexW(std::ptr::null(), 1, wide.as_ptr()) };
    if handle.is_null() {
        return InstanceClaim::Owner;
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        InstanceClaim::FocusExisting
    } else {
        InstanceClaim::Owner
    }
}

fn boot_facade() -> AppFacade {
    let args: Vec<String> = std::env::args().collect();
    let claim = {
        #[cfg(windows)]
        {
            try_windows_single_instance()
        }
        #[cfg(not(windows))]
        {
            claim_process_instance()
        }
    };
    let data_dir = std::env::var("RESIDENTIAL_MONITOR_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join(crate::identity::IDENTIFIER));
    let _ = std::fs::create_dir_all(&data_dir);
    AppFacade::boot(data_dir, &args, claim)
}

#[tauri::command]
fn get_bootstrap(state: State<Mutex<AppFacade>>) -> Result<BootstrapDto, AppErrorDto> {
    state.lock().expect("state").bootstrap()
}

#[tauri::command]
fn subscribe_monitor(
    state: State<Mutex<AppFacade>>,
    on_event: Channel<MonitorStreamMessage>,
) -> Result<u64, AppErrorDto> {
    let message = state.lock().expect("state").subscribe();
    let id = match &message {
        MonitorStreamMessage::Bootstrap {
            subscription_id, ..
        } => *subscription_id,
        _ => 0,
    };
    on_event.send(message).map_err(|_| AppErrorDto {
        code: "channel".into(),
        message_zh: "无法发送订阅首帧。".into(),
        retryable: true,
        action: "重新订阅".into(),
        details_redacted: "channel".into(),
    })?;
    Ok(id)
}

#[tauri::command]
fn resync_monitor(
    state: State<Mutex<AppFacade>>,
    subscription_id: u64,
    on_event: Channel<MonitorStreamMessage>,
) -> Result<u64, AppErrorDto> {
    let message = state.lock().expect("state").resync(subscription_id);
    let id = match &message {
        MonitorStreamMessage::Bootstrap {
            subscription_id, ..
        } => *subscription_id,
        _ => 0,
    };
    on_event.send(message).map_err(|_| AppErrorDto {
        code: "channel".into(),
        message_zh: "无法发送 resync 首帧。".into(),
        retryable: true,
        action: "重新订阅".into(),
        details_redacted: "channel".into(),
    })?;
    Ok(id)
}

#[tauri::command]
fn query_live_connections(
    state: State<Mutex<AppFacade>>,
    query: ConnectionQuery,
) -> Result<ConnectionPage, AppErrorDto> {
    Ok(state.lock().expect("state").query(&query))
}

#[tauri::command]
fn get_connection(
    state: State<Mutex<AppFacade>>,
    identity: String,
) -> Result<Option<LiveConnectionView>, AppErrorDto> {
    Ok(state.lock().expect("state").hub.row(&identity))
}

#[tauri::command]
async fn close_connection(
    state: State<'_, Mutex<AppFacade>>,
    identity: String,
    request_id: String,
) -> Result<c2::close::CloseState, AppErrorDto> {
    let (addr, secret, connection_id) = {
        let guard = state.lock().expect("state");
        if guard.branch != c2::shell::BootBranch::NormalReady {
            return Err(AppErrorDto {
                code: "recovery_only".into(),
                message_zh: "恢复模式不能关闭连接。".into(),
                retryable: false,
                action: "先修复数据库".into(),
                details_redacted: "recovery".into(),
            });
        }
        if guard.settings.address.is_empty() {
            return Err(AppErrorDto {
                code: "not_configured".into(),
                message_zh: "尚未配置控制器。".into(),
                retryable: false,
                action: "完成设置向导".into(),
                details_redacted: "settings".into(),
            });
        }
        let addr = parse_socket(&guard.settings.address)?;
        let secret = if guard.settings.has_secret {
            guard
                .workflow
                .resolve(
                    &guard.settings.credential_target,
                    &guard.settings.secret_mode,
                )
                .ok()
                .map(|value| String::from_utf8_lossy(value.as_header_bytes()).into_owned())
        } else {
            None
        };
        let connection_id = identity
            .rsplit_once(':')
            .map(|(_, id)| id.to_string())
            .unwrap_or(identity.clone());
        (addr, secret, connection_id)
    };
    let result = {
        let session = ControllerSession::new(addr.to_string());
        session
            .close_connection(addr, secret.as_deref(), &connection_id)
            .await
            .map_err(AppErrorDto::from_status)?
    };
    let mut guard = state.lock().expect("state");
    Ok(guard.mark_close_accepted_from_control(identity, request_id, result))
}

#[tauri::command]
fn get_settings(state: State<Mutex<AppFacade>>) -> Result<ControllerSettings, AppErrorDto> {
    Ok(state.lock().expect("state").settings.clone())
}

#[tauri::command]
fn save_settings(
    state: State<Mutex<AppFacade>>,
    address: String,
    secret: Option<String>,
    session_only: bool,
) -> Result<ControllerSettings, AppErrorDto> {
    state
        .lock()
        .expect("state")
        .save_controller(address, secret, session_only)
}

#[tauri::command]
fn save_targets(state: State<Mutex<AppFacade>>, targets: Vec<String>) -> Result<u32, AppErrorDto> {
    state.lock().expect("state").save_targets(targets)
}

#[tauri::command]
fn list_routes(state: State<Mutex<AppFacade>>) -> Result<Vec<RouteDescriptor>, AppErrorDto> {
    let _ = state;
    Ok(c2::shell::default_routes())
}

#[tauri::command]
fn pick_file(
    state: State<Mutex<AppFacade>>,
    purpose: FilePurpose,
    mode: FileMode,
) -> Result<Option<String>, AppErrorDto> {
    Ok(state
        .lock()
        .expect("state")
        .pick_file(purpose, mode)
        .map(|path| path.to_string_lossy().into_owned()))
}

#[tauri::command]
fn start_operation(
    state: State<Mutex<AppFacade>>,
    operation_id: String,
    kind: String,
) -> Result<OperationProgress, AppErrorDto> {
    Ok(state
        .lock()
        .expect("state")
        .start_operation(operation_id, kind))
}

#[tauri::command]
fn cancel_operation(
    state: State<Mutex<AppFacade>>,
    operation_id: String,
) -> Result<Option<OperationProgress>, AppErrorDto> {
    Ok(state
        .lock()
        .expect("state")
        .operations
        .cancel(&operation_id))
}

#[tauri::command]
fn get_recovery_status(state: State<Mutex<AppFacade>>) -> Result<RecoveryStatus, AppErrorDto> {
    state.lock().expect("state").recovery()
}

#[tauri::command]
fn run_report(
    state: State<Mutex<AppFacade>>,
    query: ReportQuery,
) -> Result<ReportResult, AppErrorDto> {
    state.lock().expect("state").run_report(query)
}

#[tauri::command]
fn get_report(state: State<Mutex<AppFacade>>, token: String) -> Result<ReportResult, AppErrorDto> {
    state.lock().expect("state").get_report(&token)
}

#[tauri::command]
fn release_report(state: State<Mutex<AppFacade>>, token: String) -> Result<bool, AppErrorDto> {
    Ok(state.lock().expect("state").release_report(&token))
}

#[tauri::command]
fn preview_export(
    state: State<Mutex<AppFacade>>,
    token: String,
    spec: ExportSpec,
) -> Result<ExportPreview, AppErrorDto> {
    state.lock().expect("state").preview_export(&token, &spec)
}

#[tauri::command]
fn export_report(
    state: State<Mutex<AppFacade>>,
    token: String,
    spec: ExportSpec,
    path: String,
) -> Result<String, AppErrorDto> {
    state
        .lock()
        .expect("state")
        .export_report(&token, &spec, std::path::Path::new(&path))
}

#[tauri::command]
fn retention_preview(state: State<Mutex<AppFacade>>) -> Result<RetentionPreview, AppErrorDto> {
    state.lock().expect("state").retention_preview()
}

#[tauri::command]
fn run_retention(
    state: State<Mutex<AppFacade>>,
    delete: bool,
) -> Result<RetentionPreview, AppErrorDto> {
    state.lock().expect("state").run_retention(delete)
}

#[tauri::command]
fn create_backup(state: State<Mutex<AppFacade>>, path: String) -> Result<String, AppErrorDto> {
    state
        .lock()
        .expect("state")
        .create_backup(std::path::Path::new(&path))
}

#[tauri::command]
fn restore_backup(state: State<Mutex<AppFacade>>, path: String) -> Result<(), AppErrorDto> {
    state
        .lock()
        .expect("state")
        .restore_backup(std::path::Path::new(&path))
}

#[tauri::command]
fn validate_backup(state: State<Mutex<AppFacade>>, path: String) -> Result<bool, AppErrorDto> {
    state
        .lock()
        .expect("state")
        .validate_candidate(std::path::Path::new(&path))
}

#[tauri::command]
fn data_directory(state: State<Mutex<AppFacade>>) -> Result<String, AppErrorDto> {
    Ok(state
        .lock()
        .expect("state")
        .data_dir
        .to_string_lossy()
        .into_owned())
}

#[tauri::command]
fn pause_collector(state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    let mut guard = state.lock().expect("state");
    let input = guard.desktop.set_collector_running(false);
    guard.apply_lifecycle(input);
    Ok(())
}

#[tauri::command]
fn resume_collector(state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    let mut guard = state.lock().expect("state");
    let input = guard.desktop.set_collector_running(true);
    guard.apply_lifecycle(input);
    Ok(())
}

#[tauri::command]
fn reconnect_now(state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    let mut guard = state.lock().expect("state");
    let input = guard.desktop.reconnect();
    guard.apply_lifecycle(input);
    Ok(())
}

#[tauri::command]
fn notify_power_event(state: State<Mutex<AppFacade>>, sleeping: bool) -> Result<(), AppErrorDto> {
    let mut guard = state.lock().expect("state");
    let input = if sleeping {
        guard.desktop.on_sleep()
    } else {
        guard.desktop.on_resume()
    };
    guard.apply_lifecycle(input);
    Ok(())
}

#[tauri::command]
fn complete_wizard(state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    state.lock().expect("state").complete_wizard()
}

#[tauri::command]
fn shutdown_app(app: tauri::AppHandle, state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    let _ = state.lock().expect("state").shutdown();
    app.exit(0);
    Ok(())
}

#[tauri::command]
fn tray_summary(state: State<Mutex<AppFacade>>) -> Result<c2::desktop::TraySummary, AppErrorDto> {
    let guard = state.lock().expect("state");
    Ok(guard
        .desktop
        .tray_summary(&guard.hub.overview().health.session))
}

fn attach_window_close(app: &tauri::App) {
    if let Some(window) = app.get_webview_window("main") {
        let handle = app.handle().clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Some(state) = handle.try_state::<Mutex<AppFacade>>() {
                    let mut guard = state.lock().expect("state");
                    let _ = guard.desktop.close_window();
                }
                if let Some(window) = handle.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
        });
    }
}

fn build_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use tauri::tray::TrayIconBuilder;

    let open = MenuItemBuilder::with_id("open", "打开窗口").build(app)?;
    let pause = MenuItemBuilder::with_id("pause", "暂停采集").build(app)?;
    let resume = MenuItemBuilder::with_id("resume", "继续采集").build(app)?;
    let reconnect = MenuItemBuilder::with_id("reconnect", "立即重连").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&open, &pause, &resume, &reconnect, &quit])
        .build()?;
    let icon = app.default_window_icon().cloned();
    let mut builder = TrayIconBuilder::new().menu(&menu);
    if let Some(icon) = icon {
        builder = builder.icon(icon);
    }
    let handle = app.handle().clone();
    builder
        .on_menu_event(move |_tray, event| {
            let id = event.id.as_ref();
            match id {
                "open" => {
                    if let Some(state) = handle.try_state::<Mutex<AppFacade>>() {
                        state.lock().expect("state").desktop.open_window();
                    }
                    if let Some(window) = handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "pause" => {
                    if let Some(state) = handle.try_state::<Mutex<AppFacade>>() {
                        let mut guard = state.lock().expect("state");
                        let input = guard.desktop.set_collector_running(false);
                        guard.apply_lifecycle(input);
                    }
                }
                "resume" => {
                    if let Some(state) = handle.try_state::<Mutex<AppFacade>>() {
                        let mut guard = state.lock().expect("state");
                        let input = guard.desktop.set_collector_running(true);
                        guard.apply_lifecycle(input);
                    }
                }
                "reconnect" => {
                    if let Some(state) = handle.try_state::<Mutex<AppFacade>>() {
                        let mut guard = state.lock().expect("state");
                        let input = guard.desktop.reconnect();
                        guard.apply_lifecycle(input);
                    }
                }
                "quit" => {
                    if let Some(state) = handle.try_state::<Mutex<AppFacade>>() {
                        let _ = state.lock().expect("state").shutdown();
                    }
                    handle.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let facade = boot_facade();
    if facade.desktop.instance == InstanceClaim::FocusExisting {
        return;
    }
    let background = facade.desktop.launch_mode == c2::desktop::LaunchMode::Background;
    tauri::Builder::default()
        .manage(Mutex::new(facade))
        .setup(move |app| {
            attach_window_close(app);
            let _ = build_tray(app);
            if background {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            let _ = app.emit("desktop-ready", true);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_bootstrap,
            subscribe_monitor,
            resync_monitor,
            query_live_connections,
            get_connection,
            close_connection,
            get_settings,
            save_settings,
            save_targets,
            list_routes,
            pick_file,
            start_operation,
            cancel_operation,
            get_recovery_status,
            run_report,
            get_report,
            release_report,
            preview_export,
            export_report,
            retention_preview,
            run_retention,
            create_backup,
            restore_backup,
            validate_backup,
            data_directory,
            pause_collector,
            resume_collector,
            reconnect_now,
            notify_power_event,
            complete_wizard,
            shutdown_app,
            tray_summary
        ])
        .run(tauri::generate_context!())
        .expect("启动家宽流量监控失败");
}
