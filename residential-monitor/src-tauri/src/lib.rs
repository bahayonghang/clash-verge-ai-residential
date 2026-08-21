pub mod accounting;
pub mod app_log;
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
pub mod live_table_layout;
pub mod redact;
pub mod residential;
pub mod session;
pub mod session_host;
pub mod sqlite_probe;
pub mod storage;
pub mod theme;
pub mod transport;
pub mod workload;

use crate::i18n::{health_title, t, UiLocale};
use crate::live_table_layout::LiveTableLayout;
use crate::session::ControllerSession;
#[cfg(not(windows))]
use c2::desktop::ProcessSingleInstance;
use c2::desktop::{tray_chrome, InstanceClaim, ShutdownPhase, TrayVisual};
use c2::facade::{parse_socket_locale, AppErrorDto, AppFacade, BootstrapDto, ProbeResult};
use c2::hub::{LiveConnectionView, MonitorStreamMessage};
use c2::query::{ConnectionPage, ConnectionQuery};
use c2::settings::ControllerSettings;
use c2::shell::{
    default_routes_for, BootBranch, FileMode, FilePurpose, OperationProgress, RecoveryStatus,
    RouteDescriptor,
};
use c2::subscriptions::SubscriptionRegistry;
use c3::archive::{ReportArchivePage, ReportArchiveService};
use c3::export::{ExportPreview, ExportSpec};
use c3::query::{ReportQuery, ReportResult};
use c3::retention::RetentionPreview;
use c3::share::ResidentialShare;
use c3::snapshot::ReportSnapshotStore;
use c4::diagnose::DiagnosticsSnapshot;
use c4::notify::NotifyCapability;
use c4::types::{AlertCenterPage, AlertRule, AlertSummary};
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
    use windows_sys::Win32::System::Threading::{CreateEventW, CreateMutexW, OpenEventW, SetEvent};

    let event_name = format!("{}.activate", crate::identity::IDENTIFIER);
    let event_wide: Vec<u16> = std::ffi::OsString::from(&event_name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    // Create the activation event before the mutex so a second process cannot
    // observe the owner without having an event to signal.
    let _event = unsafe { CreateEventW(std::ptr::null(), 0, 0, event_wide.as_ptr()) };

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
        let signal = unsafe { OpenEventW(0x0002, 0, event_wide.as_ptr()) };
        if !signal.is_null() {
            let _ = unsafe { SetEvent(signal) };
        }
        InstanceClaim::FocusExisting
    } else {
        // Raw Win32 handles are not closed by Rust; leaving this owner handle
        // open keeps the named event available for the listener and later
        // second-instance signals.
        InstanceClaim::Owner
    }
}

#[cfg(windows)]
fn start_windows_activation_listener(app: AppHandle) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::System::Threading::{OpenEventW, WaitForSingleObject};

    let event_name = format!("{}.activate", crate::identity::IDENTIFIER);
    let wide: Vec<u16> = std::ffi::OsString::from(&event_name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let event = unsafe { OpenEventW(0x0010_0000, 0, wide.as_ptr()) } as usize;
    if event == 0 {
        crate::app_log::emit(
            crate::app_log::Level::Warn,
            "instance_activation_listener",
            serde_json::json!({ "status": "unavailable" }),
        );
        return;
    }
    std::thread::spawn(move || loop {
        let event = event as windows_sys::Win32::Foundation::HANDLE;
        // A finite wait lets the listener stop once the owner has entered its
        // shutdown phases without holding any AppFacade lock.
        let result = unsafe { WaitForSingleObject(event, 500) };
        if result == 0 {
            open_main_window(&app);
            continue;
        }
        if result == 0x0000_0102 {
            let should_stop = app
                .try_state::<Mutex<AppFacade>>()
                .map(|state| {
                    state
                        .lock()
                        .map(|guard| guard.desktop.shutdown != ShutdownPhase::Idle)
                        .unwrap_or(true)
                })
                .unwrap_or(true);
            if should_stop {
                break;
            }
            continue;
        }
        break;
    });
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

async fn collector_loop_tick(handle: &AppHandle) -> bool {
    let Some(state) = handle.try_state::<Mutex<AppFacade>>() else {
        return false;
    };
    let plan = {
        let guard = state.lock().expect("state");
        if guard.desktop.shutdown != ShutdownPhase::Idle {
            return false;
        }
        c2::collector::plan_tick(&guard)
    };
    if plan.should_fetch {
        if let Some(addr) = plan.address() {
            let result = c2::collector::fetch_snapshot(addr, plan.secret()).await;
            let message = {
                let mut guard = state.lock().expect("state");
                if guard.desktop.shutdown != ShutdownPhase::Idle {
                    return false;
                }
                if guard.desktop.collector_running
                    && !matches!(
                        guard.session_status,
                        crate::controller::SessionStatus::Cancelled
                    )
                {
                    c2::collector::apply_tick_result(&mut guard, result)
                } else {
                    None
                }
            };
            if let Some(message) = message {
                forward_published(&state, [message]);
            }
        }
    }
    archive_tick(&state);
    sync_tray_chrome(handle);
    true
}

fn archive_tick(state: &Mutex<AppFacade>) {
    archive_tick_at(state, chrono::Utc::now().timestamp());
}

fn archive_tick_at(state: &Mutex<AppFacade>, now_utc: i64) {
    let prepared = {
        let guard = match state.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if guard.branch != BootBranch::NormalReady {
            return;
        }
        if guard.desktop.shutdown != ShutdownPhase::Idle {
            return;
        }
        let Some(storage) = guard.storage.as_ref() else {
            return;
        };
        let connection = storage.connection();
        let _ = ReportArchiveService::purge_expired(connection, now_utc);
        let job = match ReportArchiveService::next_job(connection, now_utc) {
            Ok(Some(job)) => job,
            _ => return,
        };
        (
            job,
            storage.path().to_path_buf(),
            guard.data_dir.clone(),
            guard.raw_retain_days,
        )
    };
    let (job, db_path, data_dir, raw_retain_days) = prepared;
    // 独立 spool 目录，避免新 store 清理门面里仍有效的 token。
    let mut store = ReportSnapshotStore::open(data_dir.join("archive-tick"));
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let outcome = c3::ReportService::run(
        &db_path,
        &mut store,
        job.query.clone(),
        now_utc,
        raw_retain_days,
        &cancel,
        None,
    );
    let token = outcome
        .as_ref()
        .ok()
        .map(|item| item.report_snapshot_token.clone());
    persist_archive_outcome(state, &job, outcome, now_utc);
    if let Some(token) = token {
        store.release(&token);
    }
}

fn persist_archive_outcome(
    state: &Mutex<AppFacade>,
    job: &c3::archive::ArchiveJob,
    outcome: Result<ReportResult, c3::query::ReportError>,
    now_utc: i64,
) {
    let Ok(guard) = state.lock() else {
        return;
    };
    if guard.branch != BootBranch::NormalReady || guard.desktop.shutdown != ShutdownPhase::Idle {
        return;
    }
    let Some(storage) = guard.storage.as_ref() else {
        return;
    };
    let _ = ReportArchiveService::persist_outcome(storage.connection(), job, outcome, now_utc);
}

fn boot_facade() -> Option<AppFacade> {
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
    if claim == InstanceClaim::FocusExisting {
        crate::app_log::emit(
            crate::app_log::Level::Info,
            "instance_focus_existing",
            serde_json::json!({}),
        );
        return None;
    }
    let data_dir = std::env::var("RESIDENTIAL_MONITOR_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join(crate::identity::IDENTIFIER));
    let _ = std::fs::create_dir_all(&data_dir);
    let mut facade = AppFacade::boot(data_dir, &args, claim);
    #[cfg(windows)]
    attach_windows_credentials(&mut facade);
    Some(facade)
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
    app: AppHandle,
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
            sync_tray_chrome(&app);
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
            sync_tray_chrome(&app);
            Err(AppErrorDto::from_status_locale(status, locale))
        }
    }
}

#[tauri::command]
fn disconnect_controller(
    app: AppHandle,
    state: State<Mutex<AppFacade>>,
) -> Result<ProbeResult, AppErrorDto> {
    let (message, locale) = {
        let mut guard = state.lock().expect("state");
        let message = guard.disconnect_now();
        (message, guard.ui_locale)
    };
    if let Some(message) = message {
        forward_published(&state, [message]);
    }
    sync_tray_chrome(&app);
    Ok(AppFacade::probe_result_locale(
        crate::controller::SessionStatus::Cancelled,
        locale,
    ))
}

#[tauri::command]
fn list_routes(state: State<Mutex<AppFacade>>) -> Result<Vec<RouteDescriptor>, AppErrorDto> {
    let locale = state.lock().expect("state").ui_locale;
    Ok(default_routes_for(locale))
}

#[tauri::command]
fn save_ui_locale(
    app: AppHandle,
    state: State<Mutex<AppFacade>>,
    locale: String,
) -> Result<String, AppErrorDto> {
    let parsed = state.lock().expect("state").save_ui_locale(&locale)?;
    apply_locale_chrome(&app, parsed);
    Ok(parsed.as_str().into())
}

#[tauri::command]
fn save_ui_theme(state: State<Mutex<AppFacade>>, theme: String) -> Result<String, AppErrorDto> {
    let parsed = state.lock().expect("state").save_ui_theme(&theme)?;
    Ok(parsed.as_str().into())
}

#[tauri::command]
fn save_ui_font(state: State<Mutex<AppFacade>>, font: String) -> Result<String, AppErrorDto> {
    let parsed = state.lock().expect("state").save_ui_font(&font)?;
    Ok(parsed.as_str().to_string())
}

#[tauri::command]
fn list_ui_fonts(state: State<Mutex<AppFacade>>) -> Result<Vec<String>, AppErrorDto> {
    crate::theme::list_installed_families().map_err(|_| {
        state
            .lock()
            .expect("state")
            .err("io", "error.font_list", "action.retry", true)
    })
}

#[tauri::command]
fn save_ui_font_size(state: State<Mutex<AppFacade>>, size: String) -> Result<String, AppErrorDto> {
    let parsed = state.lock().expect("state").save_ui_font_size(&size)?;
    Ok(parsed.as_str().into())
}

#[tauri::command]
fn save_ui_density(state: State<Mutex<AppFacade>>, density: String) -> Result<String, AppErrorDto> {
    let parsed = state.lock().expect("state").save_ui_density(&density)?;
    Ok(parsed.as_str().into())
}

#[tauri::command]
fn save_ui_sidebar_width(state: State<Mutex<AppFacade>>, width: i32) -> Result<i32, AppErrorDto> {
    state.lock().expect("state").save_ui_sidebar_width(width)
}

#[tauri::command]
fn save_live_table_layout(
    state: State<Mutex<AppFacade>>,
    layout: LiveTableLayout,
) -> Result<LiveTableLayout, AppErrorDto> {
    state.lock().expect("state").save_live_table_layout(layout)
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
    persist_manual: Option<bool>,
) -> Result<ReportResult, AppErrorDto> {
    state
        .lock()
        .expect("state")
        .run_report(query, persist_manual.unwrap_or(false))
}

#[tauri::command]
fn residential_share(
    state: State<Mutex<AppFacade>>,
    range_start_utc: i64,
    range_end_utc: i64,
    display_timezone: String,
) -> Result<ResidentialShare, AppErrorDto> {
    state
        .lock()
        .expect("state")
        .residential_share(range_start_utc, range_end_utc, display_timezone)
}

#[tauri::command]
fn list_report_archives(
    state: State<Mutex<AppFacade>>,
    kind: Option<String>,
    after: Option<String>,
    limit: Option<u32>,
) -> Result<ReportArchivePage, AppErrorDto> {
    state
        .lock()
        .expect("state")
        .list_report_archives(kind, after, limit)
}

#[tauri::command]
fn get_report_archive(
    state: State<Mutex<AppFacade>>,
    archive_id: String,
) -> Result<ReportResult, AppErrorDto> {
    state.lock().expect("state").get_report_archive(&archive_id)
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
fn pause_collector(app: AppHandle, state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    let message = {
        let mut guard = state.lock().expect("state");
        guard.pause_collector()
    };
    if let Some(message) = message {
        forward_published(&state, [message]);
    }
    sync_tray_chrome(&app);
    Ok(())
}

#[tauri::command]
fn resume_collector(app: AppHandle, state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    let message = state.lock().expect("state").resume_collector();
    if let Some(message) = message {
        forward_published(&state, [message]);
    }
    sync_tray_chrome(&app);
    Ok(())
}

#[tauri::command]
fn reconnect_now(app: AppHandle, state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    let message = state.lock().expect("state").reconnect_now();
    if let Some(message) = message {
        forward_published(&state, [message]);
    }
    sync_tray_chrome(&app);
    Ok(())
}

#[tauri::command]
fn notify_power_event(
    app: AppHandle,
    state: State<Mutex<AppFacade>>,
    sleeping: bool,
) -> Result<(), AppErrorDto> {
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
    sync_tray_chrome(&app);
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
fn open_log_dir(state: State<Mutex<AppFacade>>) -> Result<String, AppErrorDto> {
    state.lock().expect("state").open_log_dir()
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

fn open_main_window(app: &AppHandle) {
    if let Some(state) = app.try_state::<Mutex<AppFacade>>() {
        let message = state.lock().expect("state").open_main_window();
        if let Some(message) = message {
            forward_published(&state, [message]);
        }
    }
    sync_tray_chrome(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn tray_icon_image(visual: TrayVisual) -> tauri::image::Image<'static> {
    match visual {
        TrayVisual::Collecting => tauri::include_image!("icons/tray-collecting.png"),
        TrayVisual::Connecting => tauri::include_image!("icons/tray-connecting.png"),
        TrayVisual::Paused => tauri::include_image!("icons/tray-paused.png"),
        TrayVisual::Fault => tauri::include_image!("icons/tray-fault.png"),
    }
}

fn tray_tooltip(locale: UiLocale, session: &str) -> String {
    let product = t(locale, "product.display_name");
    let title = health_title(locale, session);
    if title.is_empty() {
        format!("{product} — {session}")
    } else {
        format!("{product} — {title}")
    }
}

fn build_tray_menu<R: Runtime>(
    app: &impl Manager<R>,
    locale: UiLocale,
    collector_running: bool,
) -> Result<Menu<R>, Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    let open = MenuItemBuilder::with_id("open", t(locale, "tray.open")).build(app)?;
    let pause = MenuItemBuilder::with_id("pause", t(locale, "tray.pause"))
        .enabled(collector_running)
        .build(app)?;
    let resume = MenuItemBuilder::with_id("resume", t(locale, "tray.resume"))
        .enabled(!collector_running)
        .build(app)?;
    let reconnect =
        MenuItemBuilder::with_id("reconnect", t(locale, "tray.reconnect")).build(app)?;
    let quit = MenuItemBuilder::with_id("quit", t(locale, "tray.quit")).build(app)?;
    Ok(MenuBuilder::new(app)
        .items(&[&open, &pause, &resume, &reconnect, &quit])
        .build()?)
}

fn sync_tray_chrome(app: &AppHandle) {
    let Some(state) = app.try_state::<Mutex<AppFacade>>() else {
        return;
    };
    let (visual, running, locale, skip_icon, skip_menu, skip_tooltip, tooltip) = {
        let mut guard = state.lock().expect("state");
        let health = guard.hub.overview().health;
        let chrome = tray_chrome(
            guard.desktop.collector_running,
            &health.session,
            health.storage_ok,
        );
        let running = guard.desktop.collector_running;
        let locale = guard.ui_locale;
        let tooltip = tray_tooltip(locale, &chrome.tooltip_session);
        let skip_icon = guard.desktop.last_tray_visual == Some(chrome.visual);
        let skip_menu = guard.desktop.last_tray_running == Some(running);
        let skip_tooltip = guard.desktop.last_tray_tooltip.as_deref() == Some(tooltip.as_str());
        if !skip_icon {
            guard.desktop.last_tray_visual = Some(chrome.visual);
        }
        if !skip_menu {
            guard.desktop.last_tray_running = Some(running);
        }
        if !skip_tooltip {
            guard.desktop.last_tray_tooltip = Some(tooltip.clone());
        }
        (
            chrome.visual,
            running,
            locale,
            skip_icon,
            skip_menu,
            skip_tooltip,
            tooltip,
        )
    };
    let Some(tray) = app.tray_by_id("main") else {
        return;
    };
    if !skip_icon {
        let _ = tray.set_icon(Some(tray_icon_image(visual)));
    }
    if !skip_tooltip {
        let _ = tray.set_tooltip(Some(tooltip));
    }
    if !skip_menu {
        if let Ok(menu) = build_tray_menu(app, locale, running) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn apply_locale_chrome(app: &AppHandle, locale: UiLocale) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(t(locale, "product.display_name"));
    }
    if let Some(state) = app.try_state::<Mutex<AppFacade>>() {
        let mut guard = state.lock().expect("state");
        guard.desktop.last_tray_running = None;
        guard.desktop.last_tray_tooltip = None;
    }
    sync_tray_chrome(app);
}

fn build_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let (locale, running, visual, tooltip) = {
        let state = app.state::<Mutex<AppFacade>>();
        let guard = state.lock().expect("state");
        let health = guard.hub.overview().health;
        let chrome = tray_chrome(
            guard.desktop.collector_running,
            &health.session,
            health.storage_ok,
        );
        (
            guard.ui_locale,
            guard.desktop.collector_running,
            chrome.visual,
            tray_tooltip(guard.ui_locale, &chrome.tooltip_session),
        )
    };
    let handle = app.handle().clone();
    let menu = build_tray_menu(&handle, locale, running)?;
    let icon = tray_icon_image(visual);
    let handle_menu = app.handle().clone();
    let handle_click = app.handle().clone();
    TrayIconBuilder::with_id("main")
        .menu(&menu)
        .icon(icon)
        .tooltip(tooltip)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(move |_tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            }
            | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => open_main_window(&handle_click),
            _ => {}
        })
        .on_menu_event(move |_tray, event| {
            let id = event.id.as_ref();
            match id {
                "open" => open_main_window(&handle_menu),
                "pause" => {
                    if let Some(state) = handle_menu.try_state::<Mutex<AppFacade>>() {
                        let message = {
                            let mut guard = state.lock().expect("state");
                            guard.pause_collector()
                        };
                        if let Some(message) = message {
                            forward_published(&state, [message]);
                        }
                    }
                    sync_tray_chrome(&handle_menu);
                }
                "resume" => {
                    if let Some(state) = handle_menu.try_state::<Mutex<AppFacade>>() {
                        let message = state.lock().expect("state").resume_collector();
                        if let Some(message) = message {
                            forward_published(&state, [message]);
                        }
                    }
                    sync_tray_chrome(&handle_menu);
                }
                "reconnect" => {
                    if let Some(state) = handle_menu.try_state::<Mutex<AppFacade>>() {
                        let message = state.lock().expect("state").reconnect_now();
                        if let Some(message) = message {
                            forward_published(&state, [message]);
                        }
                    }
                    sync_tray_chrome(&handle_menu);
                }
                "quit" => {
                    if let Some(state) = handle_menu.try_state::<Mutex<AppFacade>>() {
                        let _ = state.lock().expect("state").shutdown();
                    }
                    handle_menu.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    crate::app_log::init();
    let Some(facade) = boot_facade() else {
        return;
    };
    let launch = match facade.desktop.launch_mode {
        c2::desktop::LaunchMode::Background => "background",
        c2::desktop::LaunchMode::Interactive => "interactive",
    };
    let branch = match facade.branch {
        c2::shell::BootBranch::NormalReady => "normal-ready",
        c2::shell::BootBranch::RecoveryOnly => "recovery-only",
    };
    crate::app_log::emit(
        crate::app_log::Level::Info,
        "boot",
        serde_json::json!({
            "launch": launch,
            "branch": branch,
            "version": env!("CARGO_PKG_VERSION"),
        }),
    );
    let background = facade.desktop.launch_mode == c2::desktop::LaunchMode::Background;
    tauri::Builder::default()
        .manage(Mutex::new(facade))
        .setup(move |app| {
            attach_window_close(app);
            #[cfg(windows)]
            start_windows_activation_listener(app.handle().clone());
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
                    if !collector_loop_tick(&handle).await {
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
            save_ui_theme,
            save_ui_font,
            list_ui_fonts,
            save_ui_font_size,
            save_ui_density,
            save_ui_sidebar_width,
            save_live_table_layout,
            save_targets,
            test_controller,
            disconnect_controller,
            list_routes,
            pick_file,
            start_operation,
            cancel_operation,
            get_recovery_status,
            run_report,
            residential_share,
            list_report_archives,
            get_report_archive,
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
            open_log_dir,
            preview_delete_local_data,
            confirm_delete_local_data,
            run_user_vacuum
        ])
        .run(tauri::generate_context!())
        .expect("启动家宽流量监控失败");
}

#[cfg(test)]
mod archive_scheduler_tests {
    use super::*;
    use c2::desktop::{InstanceClaim, ShutdownPhase};
    use tempfile::tempdir;

    fn list_kind(state: &Mutex<AppFacade>, kind: &str) -> usize {
        let guard = state.lock().expect("state");
        let storage = guard.storage.as_ref().expect("storage");
        ReportArchiveService::list(storage.connection(), Some(kind), None, None)
            .expect("list")
            .items
            .iter()
            .filter(|item| item.status == "ok")
            .count()
    }

    #[test]
    fn archive_tick_skips_recovery_and_shutdown() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.branch = BootBranch::RecoveryOnly;
        let state = Mutex::new(facade);
        archive_tick_at(&state, chrono::Utc::now().timestamp());
        assert_eq!(list_kind(&state, "hour"), 0);

        let dir = tempdir().expect("dir2");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.desktop.shutdown = ShutdownPhase::StopIntake;
        let state = Mutex::new(facade);
        archive_tick_at(&state, chrono::Utc::now().timestamp());
        assert_eq!(list_kind(&state, "hour"), 0);
    }

    #[test]
    fn archive_tick_writes_closed_hour_once_then_day() {
        let dir = tempdir().expect("dir");
        let facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        let state = Mutex::new(facade);
        let now = chrono::Utc::now().timestamp();
        archive_tick_at(&state, now);
        assert_eq!(list_kind(&state, "hour"), 1);
        assert_eq!(list_kind(&state, "day"), 0);
        archive_tick_at(&state, now);
        assert_eq!(list_kind(&state, "hour"), 1);
        assert_eq!(list_kind(&state, "day"), 1);
        archive_tick_at(&state, now);
        assert_eq!(list_kind(&state, "hour"), 2);
        assert_eq!(list_kind(&state, "day"), 1);
    }
}
