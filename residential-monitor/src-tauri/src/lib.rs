pub mod accounting;
pub mod bench;
pub mod c0_contract;
pub mod c2;
pub mod c3;
pub mod c4;
pub mod c5;
pub mod candidate_schema;
pub mod controller;
pub mod credential;
pub mod evidence;
pub mod i18n;
pub mod identity;
pub mod live;
pub mod session;
pub mod sqlite_probe;
pub mod storage;
pub mod transport;
pub mod workload;

use crate::session::ControllerSession;
#[cfg(not(windows))]
use c2::desktop::ProcessSingleInstance;
use c2::desktop::{InstanceClaim, ShutdownPhase};
use c2::facade::{parse_socket_locale, AppErrorDto, AppFacade, BootstrapDto, ProbeResult};
use c2::hub::{LiveConnectionView, MonitorStreamMessage};
use c2::query::{ConnectionPage, ConnectionQuery};
use c2::settings::ControllerSettings;
use c2::shell::{FileMode, FilePurpose, OperationProgress, RecoveryStatus, RouteDescriptor};
use c2::subscriptions::SubscriptionRegistry;
use c3::export::{ExportPreview, ExportSpec};
use c3::query::{ReportQuery, ReportResult};
use c3::retention::RetentionPreview;
use c4::diagnose::DiagnosticsSnapshot;
use c4::notify::NotifyCapability;
use c4::types::{AlertCenterPage, AlertRule, AlertSummary};
use crate::i18n::{t, UiLocale};
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::menu::Menu;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

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

fn live_channels() -> &'static Mutex<SubscriptionRegistry<Channel<MonitorStreamMessage>>> {
    static CHANNELS: std::sync::OnceLock<
        Mutex<SubscriptionRegistry<Channel<MonitorStreamMessage>>>,
    > = std::sync::OnceLock::new();
    CHANNELS.get_or_init(|| Mutex::new(SubscriptionRegistry::new()))
}

fn forward_published(
    state: &Mutex<AppFacade>,
    messages: impl IntoIterator<Item = MonitorStreamMessage>,
) {
    let mut dead = Vec::new();
    {
        let mut registry = live_channels().lock().expect("channels");
        for message in messages {
            dead.extend(registry.forward(&message));
        }
    }
    if dead.is_empty() {
        return;
    }
    let guard = state.lock().expect("state");
    for id in dead {
        guard.hub.drop_subscription(id);
    }
}

async fn collector_loop_tick(state: &Mutex<AppFacade>) -> bool {
    let plan = {
        let guard = state.lock().expect("state");
        if guard.desktop.shutdown != ShutdownPhase::Idle {
            return false;
        }
        c2::collector::plan_tick(&guard)
    };
    if !plan.should_fetch {
        return true;
    }
    let Some(addr) = plan.address() else {
        return true;
    };
    let result = c2::collector::fetch_snapshot(addr, plan.secret()).await;
    let message = {
        let mut guard = state.lock().expect("state");
        if guard.desktop.shutdown != ShutdownPhase::Idle {
            return false;
        }
        if !guard.desktop.collector_running
            || matches!(
                guard.session_status,
                crate::controller::SessionStatus::Cancelled
            )
        {
            return true;
        }
        c2::collector::apply_tick_result(&mut guard, result)
    };
    if let Some(message) = message {
        forward_published(state, [message]);
    }
    true
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
    let mut facade = AppFacade::boot(data_dir, &args, claim);
    #[cfg(windows)]
    attach_windows_credentials(&mut facade);
    facade
}

#[cfg(windows)]
fn attach_windows_credentials(facade: &mut AppFacade) {
    if facade.branch != c2::shell::BootBranch::NormalReady {
        return;
    }
    facade.workflow = c2::settings::SettingsWorkflow::new(
        crate::credential::windows_cm::WindowsCredentialManager,
        true,
    );
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
    live_channels()
        .lock()
        .expect("channels")
        .insert(id, on_event);
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
    {
        let mut registry = live_channels().lock().expect("channels");
        registry.remove(subscription_id);
        on_event.send(message).map_err(|_| AppErrorDto {
            code: "channel".into(),
            message_zh: "无法发送 resync 首帧。".into(),
            retryable: true,
            action: "重新订阅".into(),
            details_redacted: "channel".into(),
        })?;
        registry.insert(id, on_event);
    }
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
            return Err(guard.err(
                "recovery_only",
                "error.recovery_only_close",
                "action.fix_db",
                false,
            ));
        }
        if guard.settings.address.is_empty() {
            return Err(guard.err(
                "not_configured",
                "error.not_configured",
                "action.complete_wizard",
                false,
            ));
        }
        let addr = parse_socket_locale(&guard.settings.address, guard.ui_locale)?;
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
            .map_err(|status| {
                let locale = state.lock().expect("state").ui_locale;
                AppErrorDto::from_status_locale(status, locale)
            })?
    };
    let mut guard = state.lock().expect("state");
    Ok(guard.mark_close_accepted_from_control(identity, request_id, result))
}

#[tauri::command]
fn get_settings(state: State<Mutex<AppFacade>>) -> Result<ControllerSettings, AppErrorDto> {
    Ok(state.lock().expect("state").settings.clone())
}

#[tauri::command]
fn get_controller_secret(state: State<Mutex<AppFacade>>) -> Result<Option<String>, AppErrorDto> {
    state.lock().expect("state").reveal_secret()
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
async fn test_controller(
    state: State<'_, Mutex<AppFacade>>,
    address: String,
    secret: Option<String>,
) -> Result<ProbeResult, AppErrorDto> {
    {
        let mut guard = state.lock().expect("state");
        if guard.branch != c2::shell::BootBranch::NormalReady {
            return Err(guard.err(
                "recovery_only",
                "error.recovery_only_probe",
                "action.fix_db",
                false,
            ));
        }
        guard.save_controller(address.clone(), secret, false)?;
    }
    let (addr, secret) = {
        let guard = state.lock().expect("state");
        let addr = parse_socket_locale(&guard.settings.address, guard.ui_locale)?;
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
        (addr, secret)
    };
    let mut session = ControllerSession::new(addr.to_string());
    match session.connect_tcp(addr, secret.as_deref()).await {
        Ok(inputs) => {
            let messages = {
                let mut guard = state.lock().expect("state");
                guard.session.endpoint = addr.to_string();
                guard.session.core_identity = session.core_identity;
                guard.apply_probe_ok(inputs)
            };
            forward_published(&state, messages);
            let locale = state.lock().expect("state").ui_locale;
            Ok(AppFacade::probe_result_locale(
                crate::controller::SessionStatus::Connected,
                locale,
            ))
        }
        Err(status) => {
            let (message, locale) = {
                let mut guard = state.lock().expect("state");
                let message = guard.apply_probe_err(status);
                (message, guard.ui_locale)
            };
            if let Some(message) = message {
                forward_published(&state, [message]);
            }
            Err(AppErrorDto::from_status_locale(status, locale))
        }
    }
}

#[tauri::command]
fn disconnect_controller(state: State<Mutex<AppFacade>>) -> Result<ProbeResult, AppErrorDto> {
    let (message, locale) = {
        let mut guard = state.lock().expect("state");
        let message = guard.disconnect_now();
        (message, guard.ui_locale)
    };
    if let Some(message) = message {
        forward_published(&state, [message]);
    }
    Ok(AppFacade::probe_result_locale(
        crate::controller::SessionStatus::Cancelled,
        locale,
    ))
}

#[tauri::command]
fn list_routes(state: State<Mutex<AppFacade>>) -> Result<Vec<RouteDescriptor>, AppErrorDto> {
    let locale = state.lock().expect("state").ui_locale;
    Ok(c2::shell::default_routes_for(locale))
}

#[tauri::command]
fn save_ui_locale(app: AppHandle, state: State<Mutex<AppFacade>>, locale: String) -> Result<String, AppErrorDto> {
    let parsed = state.lock().expect("state").save_ui_locale(&locale)?;
    apply_locale_chrome(&app, parsed);
    Ok(parsed.as_str().into())
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
    let message = {
        let mut guard = state.lock().expect("state");
        let input = guard.desktop.set_collector_running(false);
        guard.apply_lifecycle(input)
    };
    if let Some(message) = message {
        forward_published(&state, [message]);
    }
    Ok(())
}

#[tauri::command]
fn resume_collector(state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    let message = state.lock().expect("state").resume_collector();
    if let Some(message) = message {
        forward_published(&state, [message]);
    }
    Ok(())
}

#[tauri::command]
fn reconnect_now(state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    let message = state.lock().expect("state").reconnect_now();
    if let Some(message) = message {
        forward_published(&state, [message]);
    }
    Ok(())
}

#[tauri::command]
fn notify_power_event(state: State<Mutex<AppFacade>>, sleeping: bool) -> Result<(), AppErrorDto> {
    let message = {
        let mut guard = state.lock().expect("state");
        let input = if sleeping {
            guard.desktop.on_sleep()
        } else {
            guard.desktop.on_resume()
        };
        guard.apply_lifecycle(input)
    };
    if let Some(message) = message {
        forward_published(&state, [message]);
    }
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
fn list_alert_rules(state: State<Mutex<AppFacade>>) -> Result<Vec<AlertRule>, AppErrorDto> {
    state.lock().expect("state").list_alert_rules()
}

#[tauri::command]
fn upsert_alert_rule(
    state: State<Mutex<AppFacade>>,
    rule: AlertRule,
) -> Result<AlertRule, AppErrorDto> {
    state.lock().expect("state").upsert_alert_rule(rule)
}

#[tauri::command]
fn list_alert_center(
    state: State<Mutex<AppFacade>>,
    status: Option<String>,
    after: Option<String>,
) -> Result<AlertCenterPage, AppErrorDto> {
    state
        .lock()
        .expect("state")
        .list_alert_center(status, after)
}

#[tauri::command]
fn alert_summary(state: State<Mutex<AppFacade>>) -> Result<AlertSummary, AppErrorDto> {
    state.lock().expect("state").alert_summary()
}

#[tauri::command]
fn test_notification(state: State<Mutex<AppFacade>>) -> Result<NotifyCapability, AppErrorDto> {
    state.lock().expect("state").test_notification()
}

#[tauri::command]
fn get_diagnostics(state: State<Mutex<AppFacade>>) -> Result<DiagnosticsSnapshot, AppErrorDto> {
    state.lock().expect("state").get_diagnostics()
}

#[tauri::command]
fn export_diagnostics(state: State<Mutex<AppFacade>>, path: String) -> Result<String, AppErrorDto> {
    state
        .lock()
        .expect("state")
        .export_diagnostics(std::path::Path::new(&path))
}

#[tauri::command]
fn scan_outbox(state: State<Mutex<AppFacade>>) -> Result<u32, AppErrorDto> {
    state.lock().expect("state").scan_outbox()
}

#[tauri::command]
fn get_about(state: State<Mutex<AppFacade>>) -> Result<c5::AboutDto, AppErrorDto> {
    Ok(state.lock().expect("state").about())
}

#[tauri::command]
fn open_releases() -> Result<String, AppErrorDto> {
    Ok(crate::identity::RELEASES_URL.to_string())
}

#[tauri::command]
fn preview_delete_local_data(
    state: State<Mutex<AppFacade>>,
) -> Result<c5::DeletePreview, AppErrorDto> {
    Ok(state.lock().expect("state").preview_delete_local_data())
}

#[tauri::command]
fn confirm_delete_local_data(
    state: State<Mutex<AppFacade>>,
    phrase: String,
) -> Result<c5::DeleteReport, AppErrorDto> {
    state
        .lock()
        .expect("state")
        .confirm_delete_local_data(&phrase)
}

#[tauri::command]
fn run_user_vacuum(state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    state.lock().expect("state").run_user_vacuum()
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

fn build_tray_menu<R: Runtime>(
    app: &impl Manager<R>,
    locale: UiLocale,
) -> Result<Menu<R>, Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    let open = MenuItemBuilder::with_id("open", t(locale, "tray.open")).build(app)?;
    let pause = MenuItemBuilder::with_id("pause", t(locale, "tray.pause")).build(app)?;
    let resume = MenuItemBuilder::with_id("resume", t(locale, "tray.resume")).build(app)?;
    let reconnect = MenuItemBuilder::with_id("reconnect", t(locale, "tray.reconnect")).build(app)?;
    let quit = MenuItemBuilder::with_id("quit", t(locale, "tray.quit")).build(app)?;
    Ok(MenuBuilder::new(app)
        .items(&[&open, &pause, &resume, &reconnect, &quit])
        .build()?)
}

fn apply_locale_chrome(app: &AppHandle, locale: UiLocale) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(t(locale, "product.display_name"));
    }
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(menu) = build_tray_menu(app, locale) {
            let _ = tray.set_menu(Some(menu));
        }
        let _ = tray.set_tooltip(Some(t(locale, "product.display_name")));
    }
}

fn build_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::tray::TrayIconBuilder;

    let locale = app
        .state::<Mutex<AppFacade>>()
        .lock()
        .map(|guard| guard.ui_locale)
        .unwrap_or(UiLocale::Zh);
    let handle = app.handle().clone();
    let menu = build_tray_menu(&handle, locale)?;
    let icon = app.default_window_icon().cloned();
    let mut builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip(t(locale, "product.display_name"));
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
                        let message = {
                            let mut guard = state.lock().expect("state");
                            let input = guard.desktop.set_collector_running(false);
                            guard.apply_lifecycle(input)
                        };
                        if let Some(message) = message {
                            forward_published(&state, [message]);
                        }
                    }
                }
                "resume" => {
                    if let Some(state) = handle.try_state::<Mutex<AppFacade>>() {
                        let message = state.lock().expect("state").resume_collector();
                        if let Some(message) = message {
                            forward_published(&state, [message]);
                        }
                    }
                }
                "reconnect" => {
                    if let Some(state) = handle.try_state::<Mutex<AppFacade>>() {
                        let message = state.lock().expect("state").reconnect_now();
                        if let Some(message) = message {
                            forward_published(&state, [message]);
                        }
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
            let locale = app
                .state::<Mutex<AppFacade>>()
                .lock()
                .map(|guard| guard.ui_locale)
                .unwrap_or(UiLocale::Zh);
            apply_locale_chrome(app.handle(), locale);
            if background {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            if let Some(window) = app.get_webview_window("main") {
                let icon = tauri::include_image!("icons/icon.png");
                let _ = window.set_icon(icon);
            }
            let _ = app.emit("desktop-ready", true);
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(
                        crate::c2::contract::SAMPLE_INTERVAL_MS,
                    ))
                    .await;
                    let Some(state) = handle.try_state::<Mutex<AppFacade>>() else {
                        break;
                    };
                    if !collector_loop_tick(&state).await {
                        break;
                    }
                }
            });
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
            get_controller_secret,
            save_settings,
            save_ui_locale,
            save_targets,
            test_controller,
            disconnect_controller,
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
            tray_summary,
            list_alert_rules,
            upsert_alert_rule,
            list_alert_center,
            alert_summary,
            test_notification,
            get_diagnostics,
            export_diagnostics,
            scan_outbox,
            get_about,
            open_releases,
            preview_delete_local_data,
            confirm_delete_local_data,
            run_user_vacuum
        ])
        .run(tauri::generate_context!())
        .expect("启动家宽流量监控失败");
}
