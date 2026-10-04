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
pub mod data_dir;
pub mod dbcli;
pub mod dimension_rank_table_layout;
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

use crate::dimension_rank_table_layout::DimensionRankTableLayout;
use crate::i18n::{health_title, t, UiLocale};
use crate::live_table_layout::LiveTableLayout;
use crate::session::ControllerSession;
#[cfg(not(windows))]
use c2::desktop::ProcessSingleInstance;
use c2::desktop::{
    tray_chrome, AutostartError, AutostartPort, InstanceClaim, ShutdownPhase, TauriAutostartPort,
    TrayVisual,
};
use c2::dialog::TauriFileDialog;
use c2::facade::{
    lock_facade, parse_socket_locale, AppErrorDto, AppFacade, BootstrapDto, ProbeResult,
};
use c2::hub::{LiveConnectionView, MonitorStreamMessage};
use c2::query::{ConnectionPage, ConnectionQuery};
use c2::settings::{apply_autostart, ControllerSettings};
use c2::shell::{
    default_routes_for, BootBranch, FileDialogPort, FileMode, FilePurpose, OperationProgress,
    RecoveryStatus, RouteDescriptor,
};
use c2::subscriptions::SubscriptionRegistry;
use c3::archive::{ReportArchivePage, ReportArchiveService};
use c3::export::{ExportPreview, ExportSpec, HtmlDocument};
use c3::query::{ReportQuery, ReportResult};
use c3::retention::RetentionPreview;
use c3::share::ResidentialShare;
use c4::diagnose::DiagnosticsSnapshot;
use c4::notify::NotifyCapability;
use c4::types::{AlertCenterPage, AlertRule, AlertSummary};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri::menu::Menu;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutostartStateDto {
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowVisibilityDto {
    pub visible: bool,
}

#[derive(Default)]
struct WindowVisibilityState(Mutex<Option<WindowVisibilityDto>>);

impl WindowVisibilityState {
    fn changed(&self, value: WindowVisibilityDto) -> bool {
        let Ok(mut previous) = self.0.lock() else {
            return true;
        };
        if *previous == Some(value) {
            return false;
        }
        *previous = Some(value);
        true
    }
}

fn visible_for_display(is_visible: bool, is_minimized: bool) -> WindowVisibilityDto {
    WindowVisibilityDto {
        visible: is_visible && !is_minimized,
    }
}

#[tauri::command]
fn get_window_visibility(app: AppHandle) -> WindowVisibilityDto {
    actual_window_visibility(&app)
}

fn actual_window_visibility(app: &AppHandle) -> WindowVisibilityDto {
    app.get_webview_window("main")
        .map_or(WindowVisibilityDto { visible: false }, |window| {
            visible_for_display(
                window.is_visible().unwrap_or(false),
                window.is_minimized().unwrap_or(true),
            )
        })
}

fn publish_window_visibility(app: &AppHandle) {
    let value = actual_window_visibility(app);
    if let Some(state) = app.try_state::<WindowVisibilityState>() {
        if !state.changed(value) {
            return;
        }
    }
    let _ = app.emit("window-visibility", value);
}

fn map_autostart_error(
    locale: UiLocale,
    operation: &'static str,
    error: &AutostartError,
) -> AppErrorDto {
    crate::app_log::emit(
        crate::app_log::Level::Error,
        "autostart",
        serde_json::json!({ "class": error.class(), "operation": operation }),
    );
    AppErrorDto {
        code: "autostart_unavailable".into(),
        message_zh: t(locale, "error.autostart_unavailable").into(),
        retryable: true,
        action: t(locale, "action.retry_autostart").into(),
        details_redacted: "autostart_unavailable".into(),
    }
}

fn get_autostart_state_core(
    port: &dyn AutostartPort,
    locale: UiLocale,
) -> Result<AutostartStateDto, AppErrorDto> {
    port.is_enabled()
        .map(|enabled| AutostartStateDto { enabled })
        .map_err(|error| map_autostart_error(locale, "read", &error))
}

fn set_autostart_enabled_core(
    port: &dyn AutostartPort,
    locale: UiLocale,
    enabled: bool,
) -> Result<AutostartStateDto, AppErrorDto> {
    let actual = apply_autostart(port, enabled)
        .map_err(|error| map_autostart_error(locale, "write_readback", &error))?;
    crate::app_log::emit(
        crate::app_log::Level::Info,
        "autostart",
        serde_json::json!({ "class": "ok", "enabled": actual }),
    );
    Ok(AutostartStateDto { enabled: actual })
}

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

/// 只保留一个唤醒信号，耗时档案与分块维护不会阻塞 HTTP 采集线程。
struct BackgroundWorker {
    wake: std::sync::mpsc::SyncSender<()>,
    cancel: Arc<AtomicBool>,
    thread: std::thread::JoinHandle<()>,
}

impl BackgroundWorker {
    fn spawn(cancel: Arc<AtomicBool>, mut tick: impl FnMut() -> bool + Send + 'static) -> Self {
        let (wake, pending) = std::sync::mpsc::sync_channel(1);
        let stopped = Arc::clone(&cancel);
        let thread = std::thread::spawn(move || {
            while pending.recv().is_ok() {
                if stopped.load(Ordering::SeqCst) || !tick() {
                    break;
                }
            }
        });
        Self {
            wake,
            cancel,
            thread,
        }
    }

    fn signal(&self) -> bool {
        self.wake.try_send(()).is_ok()
    }

    fn stop(self) {
        self.cancel.store(true, Ordering::SeqCst);
        let _ = self.wake.try_send(());
        if self.thread.join().is_err() {
            crate::app_log::emit(
                crate::app_log::Level::Error,
                "background_worker",
                serde_json::json!({"class":"worker_panicked"}),
            );
        }
    }
}

#[derive(Default)]
struct BackgroundWorkState {
    worker: Option<BackgroundWorker>,
    pause_depth: usize,
    stopping: bool,
}

impl BackgroundWorkState {
    fn pause(&mut self, cancel: &AtomicBool) {
        self.pause_depth += 1;
        cancel.store(true, Ordering::SeqCst);
        // 持有 owner 锁直到 join 完成，避免并发维护命令越过同一个旧 worker。
        // worker 不读取 owner；这里绝不能同时持有 facade。
        if let Some(worker) = self.worker.take() {
            worker.stop();
        }
    }

    fn resume(&mut self, cancel: &AtomicBool, ready: bool) -> bool {
        self.pause_depth = self.pause_depth.saturating_sub(1);
        if self.pause_depth == 0 && !self.stopping && ready {
            cancel.store(false, Ordering::SeqCst);
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
struct BackgroundWorkOwner(Mutex<BackgroundWorkState>);

struct BackgroundWorkPause<'a>(&'a AppHandle);

impl<'a> BackgroundWorkPause<'a> {
    fn begin(app: &'a AppHandle) -> Self {
        let owner = app.state::<BackgroundWorkOwner>();
        let operations = app.state::<c2::shell::OperationRegistry>();
        owner
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pause(&operations.archive_tick_cancel());
        Self(app)
    }
}

impl Drop for BackgroundWorkPause<'_> {
    fn drop(&mut self) {
        let app = self.0;
        let state = app.state::<Mutex<AppFacade>>();
        let ready = state.lock().is_ok_and(|guard| {
            guard.branch == BootBranch::NormalReady && guard.desktop.shutdown == ShutdownPhase::Idle
        });
        let owner = app.state::<BackgroundWorkOwner>();
        let operations = app.state::<c2::shell::OperationRegistry>();
        let resume = owner
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resume(&operations.archive_tick_cancel(), ready);
        if resume {
            schedule_background_work(app);
        }
    }
}

fn schedule_background_work(app: &AppHandle) {
    let Some(owner) = app.try_state::<BackgroundWorkOwner>() else {
        return;
    };
    let Some(operations) = app.try_state::<c2::shell::OperationRegistry>() else {
        return;
    };
    let cancel = operations.archive_tick_cancel();
    if cancel.load(Ordering::SeqCst) {
        return;
    }
    let Some(state) = app.try_state::<Mutex<AppFacade>>() else {
        return;
    };
    let ready = state.lock().is_ok_and(|guard| {
        guard.branch == BootBranch::NormalReady && guard.desktop.shutdown == ShutdownPhase::Idle
    });
    if !ready {
        return;
    }
    let Ok(mut owner) = owner.0.try_lock() else {
        return;
    };
    if owner.pause_depth != 0 || owner.stopping || cancel.load(Ordering::SeqCst) {
        return;
    }
    if owner.worker.is_none() {
        let handle = app.clone();
        let mut prefer_maintenance = false;
        owner.worker = Some(BackgroundWorker::spawn(cancel, move || {
            let Some(state) = handle.try_state::<Mutex<AppFacade>>() else {
                return false;
            };
            let Ok(guard) = state.lock() else {
                return false;
            };
            if guard.desktop.shutdown != ShutdownPhase::Idle {
                return false;
            }
            if guard.branch != BootBranch::NormalReady {
                return true;
            }
            drop(guard);
            // 信号不携带旧时间；跨时区/睡眠恢复后按实际当前时间重新判定到期。
            let now = chrono::Utc::now().timestamp();
            run_fair_background_step(
                &mut prefer_maintenance,
                || archive_tick_at(&state, now),
                || match c2::facade::run_maintenance_tick(&state, now) {
                    Ok(attempted) => attempted,
                    Err(error) => {
                        crate::app_log::emit(
                            crate::app_log::Level::Error,
                            "maintenance_tick",
                            serde_json::json!({"class":error.code}),
                        );
                        true
                    }
                },
            );
            true
        }));
    }
    if let Some(worker) = owner.worker.as_ref() {
        worker.signal();
    }
}

fn run_fair_background_step(
    prefer_maintenance: &mut bool,
    mut archive: impl FnMut() -> bool,
    mut maintenance: impl FnMut() -> bool,
) {
    if *prefer_maintenance {
        if maintenance() {
            *prefer_maintenance = false;
            return;
        }
        if archive() {
            *prefer_maintenance = true;
        }
    } else {
        if archive() {
            *prefer_maintenance = true;
            return;
        }
        if maintenance() {
            *prefer_maintenance = false;
        }
    }
}

fn cancel_background_work(app: &AppHandle) {
    if let Some(operations) = app.try_state::<c2::shell::OperationRegistry>() {
        operations
            .archive_tick_cancel()
            .store(true, Ordering::SeqCst);
    }
    if let Some(owner) = app.try_state::<BackgroundWorkOwner>() {
        if let Ok(mut owner) = owner.0.lock() {
            owner.stopping = true;
            if let Some(operations) = app.try_state::<c2::shell::OperationRegistry>() {
                operations
                    .archive_tick_cancel()
                    .store(true, Ordering::SeqCst);
            }
        }
    }
}

fn stop_background_work(app: &AppHandle) {
    cancel_background_work(app);
    let worker = app.try_state::<BackgroundWorkOwner>().and_then(|owner| {
        owner
            .0
            .lock()
            .ok()
            .and_then(|mut owner| owner.worker.take())
    });
    // join 期间不能持有 facade；读查询由共享取消标志中断，writer 能完成回滚。
    if let Some(worker) = worker {
        worker.stop();
    }
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
    let Ok(guard) = state.lock() else {
        return;
    };
    for id in dead {
        guard.hub.drop_subscription(id);
    }
}

async fn collector_loop_tick(handle: &AppHandle) -> bool {
    let Some(state) = handle.try_state::<Mutex<AppFacade>>() else {
        return false;
    };
    let (plan, retried) = {
        let Ok(mut guard) = state.lock() else {
            return false;
        };
        if guard.desktop.shutdown != ShutdownPhase::Idle {
            return false;
        }
        let retried = guard.retry_writer_tick();
        (c2::collector::plan_tick(&guard), retried)
    };
    if let Some(message) = retried {
        forward_published(&state, [message]);
    }
    let message = if let Some(status) = plan.session_error() {
        let Ok(mut guard) = state.lock() else {
            return false;
        };
        if guard.desktop.shutdown != ShutdownPhase::Idle {
            return false;
        }
        if guard.desktop.collector_running
            && !matches!(
                guard.session_status,
                crate::controller::SessionStatus::Cancelled
            )
        {
            c2::collector::apply_tick_result(&mut guard, Err(status))
        } else {
            None
        }
    } else if plan.should_fetch {
        if let Some(addr) = plan.address() {
            let result = c2::collector::fetch_snapshot(addr, plan.secret()).await;
            let Ok(mut guard) = state.lock() else {
                return false;
            };
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
        } else {
            None
        }
    } else {
        None
    };
    if let Some(message) = message {
        forward_published(&state, [message]);
    }
    schedule_background_work(handle);
    sync_tray_chrome(handle);
    true
}

#[cfg(test)]
fn archive_tick(state: &Mutex<AppFacade>) -> bool {
    archive_tick_at(state, chrono::Utc::now().timestamp())
}

fn archive_tick_at(state: &Mutex<AppFacade>, now_utc: i64) -> bool {
    let prepared = {
        let mut guard = match state.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        if guard.branch != BootBranch::NormalReady {
            return false;
        }
        if guard.desktop.shutdown != ShutdownPhase::Idle {
            return false;
        }
        let Some(storage) = guard.storage.as_ref() else {
            return false;
        };
        let path = storage.path().to_path_buf();
        let Some(scheduler) = guard.archive_scheduler.take() else {
            return false;
        };
        (
            scheduler,
            path,
            guard.raw_retain_days,
            guard.operations.archive_tick_cancel(),
        )
    };
    let (mut scheduler, db_path, raw_retain_days, cancel) = prepared;
    let mut attempted = false;
    // 有界历史发现与读报告均在 facade 锁外；内部报告不创建或释放公共 token。
    match scheduler.next_job(&db_path, now_utc, &cancel) {
        Ok(Some(job)) => {
            attempted = true;
            let outcome = c3::service::run_uncached(
                &db_path,
                job.query.clone(),
                now_utc,
                raw_retain_days,
                &cancel,
                None,
            );
            let succeeded = outcome.is_ok();
            let persisted = if cancel.load(std::sync::atomic::Ordering::SeqCst) {
                false
            } else {
                persist_archive_outcome(state, &job, outcome, now_utc)
            };
            scheduler.complete(succeeded && persisted, now_utc);
        }
        Ok(None) => {}
        Err(error) => {
            attempted = true;
            log_archive_error("archive_next_job", &error);
        }
    }
    if let Ok(mut guard) = state.lock() {
        // restore/reopen 已放入新 scheduler 时，不把旧库的进度带回新 owner。
        if guard.archive_scheduler.is_none() {
            if guard.branch == BootBranch::NormalReady
                && guard.desktop.shutdown == ShutdownPhase::Idle
                && !cancel.load(Ordering::SeqCst)
                && scheduler.purge_due(now_utc)
            {
                if let Some(storage) = guard.storage.as_ref() {
                    attempted = true;
                    match ReportArchiveService::purge_expired_chunk(
                        storage.connection(),
                        now_utc,
                        &cancel,
                    ) {
                        Ok(removed) => scheduler.complete_purge(Some(removed), now_utc),
                        Err(error) => {
                            scheduler.complete_purge(None, now_utc);
                            log_archive_error("archive_purge", &error);
                        }
                    }
                }
            }
            guard.archive_scheduler = Some(scheduler);
        }
    }
    attempted
}

fn persist_archive_outcome(
    state: &Mutex<AppFacade>,
    job: &c3::archive::ArchiveJob,
    outcome: Result<ReportResult, c3::query::ReportError>,
    now_utc: i64,
) -> bool {
    let Ok(guard) = state.lock() else {
        return false;
    };
    if guard.branch != BootBranch::NormalReady
        || guard.desktop.shutdown != ShutdownPhase::Idle
        || guard.archive_scheduler.is_some()
    {
        return false;
    }
    let Some(storage) = guard.storage.as_ref() else {
        return false;
    };
    if let Ok(result) = &outcome {
        let needed = match serde_json::to_vec(result) {
            Ok(encoded) => encoded.len() as u64 + 8_192,
            Err(_) => {
                log_archive_error(
                    "archive_persist",
                    &c3::query::ReportError::Failed("encode archive"),
                );
                return false;
            }
        };
        if let Err(error) = guard.space.check(&guard.data_dir, needed) {
            log_archive_error("archive_persist", &error);
            return false;
        }
    }
    if let Err(error) =
        ReportArchiveService::persist_outcome(storage.connection(), job, outcome, now_utc)
    {
        log_archive_error("archive_persist", &error);
        return false;
    }
    true
}

fn log_archive_error(event: &'static str, error: &c3::query::ReportError) {
    crate::app_log::emit(
        crate::app_log::Level::Error,
        event,
        serde_json::json!({ "class": error.code() }),
    );
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
    let data_dir = data_dir::prepare_data_dir();
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

fn get_bootstrap_core(state: &Mutex<AppFacade>) -> Result<BootstrapDto, AppErrorDto> {
    lock_facade(state)?.bootstrap()
}

#[tauri::command]
fn get_bootstrap(state: State<Mutex<AppFacade>>) -> Result<BootstrapDto, AppErrorDto> {
    get_bootstrap_core(&state)
}

#[tauri::command]
fn subscribe_monitor(
    state: State<Mutex<AppFacade>>,
    on_event: Channel<MonitorStreamMessage>,
) -> Result<u64, AppErrorDto> {
    let message = lock_facade(&state)?.subscribe();
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
    let message = lock_facade(&state)?.resync(subscription_id);
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
    Ok(lock_facade(&state)?.query(&query))
}

#[tauri::command]
fn get_connection(
    state: State<Mutex<AppFacade>>,
    identity: String,
) -> Result<Option<LiveConnectionView>, AppErrorDto> {
    Ok(lock_facade(&state)?.hub.row(&identity))
}

#[tauri::command]
async fn close_connection(
    state: State<'_, Mutex<AppFacade>>,
    identity: String,
    request_id: String,
) -> Result<c2::close::CloseState, AppErrorDto> {
    let (addr, secret, connection_id) = {
        let guard = lock_facade(&state)?;
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
        let secret = secret.as_ref().and_then(crate::credential::Secret::as_utf8);
        session
            .close_connection(addr, secret, &connection_id)
            .await
            .map_err(|status| {
                lock_facade(&state)
                    .map(|guard| AppErrorDto::from_status_locale(status, guard.ui_locale))
                    .unwrap_or_else(|error| error)
            })?
    };
    drop(secret);
    let mut guard = lock_facade(&state)?;
    Ok(guard.mark_close_accepted_from_control(identity, request_id, result))
}

#[tauri::command]
fn get_settings(state: State<Mutex<AppFacade>>) -> Result<ControllerSettings, AppErrorDto> {
    Ok(lock_facade(&state)?.settings.clone())
}

#[tauri::command]
fn get_autostart_state(
    app: AppHandle,
    state: State<Mutex<AppFacade>>,
) -> Result<AutostartStateDto, AppErrorDto> {
    let locale = lock_facade(&state)?.ui_locale;
    let port = TauriAutostartPort::new(&app);
    get_autostart_state_core(&port, locale)
}

#[tauri::command]
fn set_autostart_enabled(
    app: AppHandle,
    state: State<Mutex<AppFacade>>,
    enabled: bool,
) -> Result<AutostartStateDto, AppErrorDto> {
    let locale = lock_facade(&state)?.ui_locale;
    let port = TauriAutostartPort::new(&app);
    set_autostart_enabled_core(&port, locale, enabled)
}

#[tauri::command]
fn get_controller_secret(state: State<Mutex<AppFacade>>) -> Result<Option<String>, AppErrorDto> {
    lock_facade(&state)?.reveal_secret()
}

#[tauri::command]
fn save_settings(
    state: State<Mutex<AppFacade>>,
    address: String,
    secret: Option<String>,
    session_only: bool,
) -> Result<ControllerSettings, AppErrorDto> {
    // 持久凭据只在探测成功后提升；设置页保存只改地址或写入 session。
    let persist_secret = if session_only { secret } else { None };
    lock_facade(&state)?.save_controller(address, persist_secret, session_only, false)
}

#[tauri::command]
fn save_targets(state: State<Mutex<AppFacade>>, targets: Vec<String>) -> Result<u32, AppErrorDto> {
    lock_facade(&state)?.save_targets(targets)
}

#[tauri::command]
async fn test_controller(
    app: AppHandle,
    state: State<'_, Mutex<AppFacade>>,
    address: String,
    secret: Option<String>,
) -> Result<ProbeResult, AppErrorDto> {
    let (result, messages) = test_controller_core(&state, address, secret).await;
    if !messages.is_empty() {
        forward_published(&state, messages);
    }
    sync_tray_chrome(&app);
    result
}

async fn test_controller_core(
    state: &Mutex<AppFacade>,
    address: String,
    secret: Option<String>,
) -> (Result<ProbeResult, AppErrorDto>, Vec<MonitorStreamMessage>) {
    let addr = {
        let guard = match lock_facade(state) {
            Ok(guard) => guard,
            Err(error) => return (Err(error), Vec::new()),
        };
        if guard.branch != c2::shell::BootBranch::NormalReady {
            return (
                Err(guard.err(
                    "recovery_only",
                    "error.recovery_only_probe",
                    "action.fix_db",
                    false,
                )),
                Vec::new(),
            );
        }
        match parse_socket_locale(&address, guard.ui_locale) {
            Ok(addr) => addr,
            Err(error) => return (Err(error), Vec::new()),
        }
    };
    let mut session = ControllerSession::new(addr.to_string());
    match session.connect_tcp(addr, secret.as_deref()).await {
        Ok(inputs) => {
            let mut guard = match lock_facade(state) {
                Ok(guard) => guard,
                Err(error) => return (Err(error), Vec::new()),
            };
            if guard.branch != c2::shell::BootBranch::NormalReady {
                return (
                    Err(guard.err(
                        "recovery_only",
                        "error.recovery_only_probe",
                        "action.fix_db",
                        false,
                    )),
                    Vec::new(),
                );
            }
            if let Err(error) = guard.save_controller(address, secret, false, true) {
                return (Err(error), Vec::new());
            }
            guard.session.endpoint = addr.to_string();
            guard.session.core_identity = session.core_identity;
            let messages = guard.apply_probe_ok(inputs);
            let locale = guard.ui_locale;
            (
                Ok(AppFacade::probe_result_locale(
                    crate::controller::SessionStatus::Connected,
                    locale,
                )),
                messages,
            )
        }
        Err(status) => {
            let mut guard = match lock_facade(state) {
                Ok(guard) => guard,
                Err(error) => return (Err(error), Vec::new()),
            };
            let message = guard.apply_probe_err(status);
            let locale = guard.ui_locale;
            (
                Err(AppErrorDto::from_status_locale(status, locale)),
                message.into_iter().collect(),
            )
        }
    }
}

#[tauri::command]
fn disconnect_controller(
    app: AppHandle,
    state: State<Mutex<AppFacade>>,
) -> Result<ProbeResult, AppErrorDto> {
    let (message, locale) = {
        let mut guard = lock_facade(&state)?;
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
    let locale = lock_facade(&state)?.ui_locale;
    Ok(default_routes_for(locale))
}

#[tauri::command]
fn save_ui_locale(
    app: AppHandle,
    state: State<Mutex<AppFacade>>,
    locale: String,
) -> Result<String, AppErrorDto> {
    let parsed = lock_facade(&state)?.save_ui_locale(&locale)?;
    apply_locale_chrome(&app, parsed);
    Ok(parsed.as_str().into())
}

#[tauri::command]
fn save_ui_theme(state: State<Mutex<AppFacade>>, theme: String) -> Result<String, AppErrorDto> {
    let parsed = lock_facade(&state)?.save_ui_theme(&theme)?;
    Ok(parsed.as_str().into())
}

#[tauri::command]
fn save_ui_font(state: State<Mutex<AppFacade>>, font: String) -> Result<String, AppErrorDto> {
    let parsed = lock_facade(&state)?.save_ui_font(&font)?;
    Ok(parsed.as_str().to_string())
}

#[tauri::command]
fn list_ui_fonts(state: State<Mutex<AppFacade>>) -> Result<Vec<String>, AppErrorDto> {
    crate::theme::list_installed_families().map_err(|_| {
        lock_facade(&state).map_or_else(
            |error| error,
            |guard| guard.err("io", "error.font_list", "action.retry", true),
        )
    })
}

#[tauri::command]
fn save_ui_font_size(state: State<Mutex<AppFacade>>, size: String) -> Result<String, AppErrorDto> {
    let parsed = lock_facade(&state)?.save_ui_font_size(&size)?;
    Ok(parsed.as_str().into())
}

#[tauri::command]
fn save_ui_density(state: State<Mutex<AppFacade>>, density: String) -> Result<String, AppErrorDto> {
    let parsed = lock_facade(&state)?.save_ui_density(&density)?;
    Ok(parsed.as_str().into())
}

#[tauri::command]
fn save_ui_sidebar_width(state: State<Mutex<AppFacade>>, width: i32) -> Result<i32, AppErrorDto> {
    lock_facade(&state)?.save_ui_sidebar_width(width)
}

#[tauri::command]
fn save_live_table_layout(
    state: State<Mutex<AppFacade>>,
    layout: LiveTableLayout,
) -> Result<LiveTableLayout, AppErrorDto> {
    lock_facade(&state)?.save_live_table_layout(layout)
}

#[tauri::command]
fn save_dimension_rank_table_layout(
    state: State<Mutex<AppFacade>>,
    layout: DimensionRankTableLayout,
) -> Result<DimensionRankTableLayout, AppErrorDto> {
    lock_facade(&state)?.save_dimension_rank_table_layout(layout)
}

#[tauri::command]
async fn pick_file(
    dialog: State<'_, Arc<dyn FileDialogPort + Send + Sync>>,
    locale: String,
    purpose: FilePurpose,
    mode: FileMode,
) -> Result<Option<String>, AppErrorDto> {
    let locale = UiLocale::parse(Some(locale.trim()));
    Ok(dialog
        .pick(locale, purpose, mode)
        .map(|path| path.to_string_lossy().into_owned()))
}

#[tauri::command]
fn start_operation(
    state: State<Mutex<AppFacade>>,
    operation_id: String,
    kind: String,
) -> Result<OperationProgress, AppErrorDto> {
    Ok(lock_facade(&state)?.start_operation(operation_id, kind))
}

#[tauri::command]
fn cancel_operation(
    state: State<c2::shell::OperationRegistry>,
    operation_id: String,
) -> Result<Option<OperationProgress>, AppErrorDto> {
    Ok(state.cancel(&operation_id))
}

#[tauri::command]
fn finish_operation(
    state: State<Mutex<AppFacade>>,
    operation_id: String,
) -> Result<Option<OperationProgress>, AppErrorDto> {
    Ok(lock_facade(&state)?.operations.finish(&operation_id))
}

#[tauri::command]
fn get_recovery_status(state: State<Mutex<AppFacade>>) -> Result<RecoveryStatus, AppErrorDto> {
    lock_facade(&state)?.recovery()
}

#[tauri::command]
fn run_report(
    state: State<Mutex<AppFacade>>,
    query: ReportQuery,
    persist_manual: Option<bool>,
    operation_id: Option<String>,
) -> Result<ReportResult, AppErrorDto> {
    c2::facade::run_report_unlocked(
        &state,
        query,
        persist_manual.unwrap_or(false),
        operation_id.as_deref(),
    )
}

#[tauri::command]
fn residential_share(
    state: State<Mutex<AppFacade>>,
    range_start_utc: i64,
    range_end_utc: i64,
    display_timezone: String,
    operation_id: Option<String>,
) -> Result<ResidentialShare, AppErrorDto> {
    c2::facade::residential_share_unlocked(
        &state,
        range_start_utc,
        range_end_utc,
        display_timezone,
        operation_id.as_deref(),
    )
}

#[tauri::command]
fn list_report_archives(
    state: State<Mutex<AppFacade>>,
    kind: Option<String>,
    after: Option<String>,
    limit: Option<u32>,
) -> Result<ReportArchivePage, AppErrorDto> {
    lock_facade(&state)?.list_report_archives(kind, after, limit)
}

#[tauri::command]
fn get_report_archive(
    state: State<Mutex<AppFacade>>,
    archive_id: String,
) -> Result<ReportResult, AppErrorDto> {
    lock_facade(&state)?.get_report_archive(&archive_id)
}

#[tauri::command]
fn get_report(state: State<Mutex<AppFacade>>, token: String) -> Result<ReportResult, AppErrorDto> {
    lock_facade(&state)?.get_report(&token)
}

#[tauri::command]
fn release_report(state: State<Mutex<AppFacade>>, token: String) -> Result<bool, AppErrorDto> {
    Ok(lock_facade(&state)?.release_report(&token))
}

#[tauri::command]
fn preview_export(
    state: State<Mutex<AppFacade>>,
    token: String,
    spec: ExportSpec,
) -> Result<ExportPreview, AppErrorDto> {
    lock_facade(&state)?.preview_export(&token, &spec)
}

#[tauri::command]
fn export_report(
    state: State<Mutex<AppFacade>>,
    token: String,
    spec: ExportSpec,
    path: String,
    operation_id: Option<String>,
) -> Result<String, AppErrorDto> {
    c2::facade::export_report_unlocked(
        &state,
        &token,
        &spec,
        std::path::Path::new(&path),
        operation_id.as_deref(),
    )
}

#[tauri::command]
fn render_report_html(
    state: State<Mutex<AppFacade>>,
    token: String,
    spec: ExportSpec,
) -> Result<HtmlDocument, AppErrorDto> {
    lock_facade(&state)?.render_report_html(&token, &spec)
}

#[tauri::command]
fn get_latest_residential_manual(
    state: State<Mutex<AppFacade>>,
) -> Result<Option<ReportResult>, AppErrorDto> {
    lock_facade(&state)?.get_latest_residential_manual()
}

#[tauri::command]
fn retention_preview(state: State<Mutex<AppFacade>>) -> Result<RetentionPreview, AppErrorDto> {
    lock_facade(&state)?.retention_preview()
}

#[tauri::command]
fn run_retention(
    state: State<Mutex<AppFacade>>,
    delete: bool,
    operation_id: Option<String>,
) -> Result<RetentionPreview, AppErrorDto> {
    c2::facade::run_retention_unlocked(&state, delete, operation_id.as_deref())
}

#[tauri::command]
fn create_backup(
    state: State<Mutex<AppFacade>>,
    path: String,
    operation_id: Option<String>,
) -> Result<String, AppErrorDto> {
    c2::facade::create_backup_unlocked(&state, std::path::Path::new(&path), operation_id.as_deref())
}

#[tauri::command]
fn restore_backup(
    app: AppHandle,
    state: State<Mutex<AppFacade>>,
    path: String,
    operation_id: Option<String>,
) -> Result<(), AppErrorDto> {
    let _pause = BackgroundWorkPause::begin(&app);
    c2::facade::restore_backup_unlocked(
        &state,
        std::path::Path::new(&path),
        operation_id.as_deref(),
    )
}

#[tauri::command]
fn validate_backup(state: State<Mutex<AppFacade>>, path: String) -> Result<bool, AppErrorDto> {
    lock_facade(&state)?.validate_candidate(std::path::Path::new(&path))
}

#[tauri::command]
fn data_directory(state: State<Mutex<AppFacade>>) -> Result<String, AppErrorDto> {
    Ok(lock_facade(&state)?.data_dir.to_string_lossy().into_owned())
}

#[tauri::command]
fn pause_collector(app: AppHandle, state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    let message = {
        let mut guard = lock_facade(&state)?;
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
    let message = lock_facade(&state)?.resume_collector();
    if let Some(message) = message {
        forward_published(&state, [message]);
    }
    sync_tray_chrome(&app);
    Ok(())
}

#[tauri::command]
fn reconnect_now(app: AppHandle, state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    let message = lock_facade(&state)?.reconnect_now();
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
        let mut guard = lock_facade(&state)?;
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
    lock_facade(&state)?.complete_wizard()
}

#[tauri::command]
fn shutdown_app(app: tauri::AppHandle, state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    cancel_background_work(&app);
    let _ = lock_facade(&state)?.shutdown();
    stop_background_work(&app);
    app.exit(0);
    Ok(())
}

#[tauri::command]
fn list_alert_rules(state: State<Mutex<AppFacade>>) -> Result<Vec<AlertRule>, AppErrorDto> {
    lock_facade(&state)?.list_alert_rules()
}

#[tauri::command]
fn upsert_alert_rule(
    state: State<Mutex<AppFacade>>,
    rule: AlertRule,
) -> Result<AlertRule, AppErrorDto> {
    lock_facade(&state)?.upsert_alert_rule(rule)
}

#[tauri::command]
fn list_alert_center(
    state: State<Mutex<AppFacade>>,
    status: Option<String>,
    after: Option<String>,
) -> Result<AlertCenterPage, AppErrorDto> {
    lock_facade(&state)?.list_alert_center(status, after)
}

#[tauri::command]
fn alert_summary(state: State<Mutex<AppFacade>>) -> Result<AlertSummary, AppErrorDto> {
    lock_facade(&state)?.alert_summary()
}

#[tauri::command]
fn test_notification(state: State<Mutex<AppFacade>>) -> Result<NotifyCapability, AppErrorDto> {
    lock_facade(&state)?.test_notification()
}

#[tauri::command]
fn get_diagnostics(state: State<Mutex<AppFacade>>) -> Result<DiagnosticsSnapshot, AppErrorDto> {
    lock_facade(&state)?.get_diagnostics()
}

#[tauri::command]
fn export_diagnostics(state: State<Mutex<AppFacade>>, path: String) -> Result<String, AppErrorDto> {
    lock_facade(&state)?.export_diagnostics(std::path::Path::new(&path))
}

#[tauri::command]
fn scan_outbox(state: State<Mutex<AppFacade>>) -> Result<u32, AppErrorDto> {
    lock_facade(&state)?.scan_outbox()
}

#[tauri::command]
fn get_about(state: State<Mutex<AppFacade>>) -> Result<c5::AboutDto, AppErrorDto> {
    Ok(lock_facade(&state)?.about())
}

#[tauri::command]
fn open_releases() -> Result<String, AppErrorDto> {
    Ok(crate::identity::RELEASES_URL.to_string())
}

#[tauri::command]
fn open_log_dir(state: State<Mutex<AppFacade>>) -> Result<String, AppErrorDto> {
    lock_facade(&state)?.open_log_dir()
}

#[tauri::command]
fn preview_delete_local_data(
    state: State<Mutex<AppFacade>>,
) -> Result<c5::DeletePreview, AppErrorDto> {
    Ok(lock_facade(&state)?.preview_delete_local_data())
}

#[tauri::command]
fn confirm_delete_local_data(
    app: AppHandle,
    state: State<Mutex<AppFacade>>,
    phrase: String,
) -> Result<c5::DeleteReport, AppErrorDto> {
    let _pause = BackgroundWorkPause::begin(&app);
    let result = {
        let mut guard = lock_facade(&state)?;
        guard.confirm_delete_local_data(&phrase)
    };
    result
}

#[tauri::command]
fn run_user_vacuum(app: AppHandle, state: State<Mutex<AppFacade>>) -> Result<(), AppErrorDto> {
    let _pause = BackgroundWorkPause::begin(&app);
    let result = {
        let mut guard = lock_facade(&state)?;
        guard.run_user_vacuum()
    };
    result
}

#[tauri::command]
fn tray_summary(state: State<Mutex<AppFacade>>) -> Result<c2::desktop::TraySummary, AppErrorDto> {
    let guard = lock_facade(&state)?;
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
                publish_window_visibility(&handle);
            } else if matches!(
                event,
                tauri::WindowEvent::Resized(_) | tauri::WindowEvent::Focused(_)
            ) {
                publish_window_visibility(&handle);
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
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    publish_window_visibility(app);
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
                    cancel_background_work(&handle_menu);
                    if let Some(state) = handle_menu.try_state::<Mutex<AppFacade>>() {
                        let _ = state.lock().expect("state").shutdown();
                    }
                    stop_background_work(&handle_menu);
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
    let operation_registry = facade.operations.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![crate::identity::AUTOSTART_ARGUMENT]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(Mutex::new(facade))
        .manage(operation_registry)
        .manage(BackgroundWorkOwner::default())
        .manage(WindowVisibilityState::default())
        .setup(move |app| {
            app.manage(Arc::new(TauriFileDialog {
                app: app.handle().clone(),
            }) as Arc<dyn FileDialogPort + Send + Sync>);
            attach_window_close(app);
            #[cfg(windows)]
            start_windows_activation_listener(app.handle().clone());
            let _ = build_tray(app);
            let locale = app
                .state::<Mutex<AppFacade>>()
                .lock()
                .map(|mut guard| {
                    guard.attach_notification_handle(app.handle().clone());
                    guard.ui_locale
                })
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
            publish_window_visibility(app.handle());
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
            get_window_visibility,
            subscribe_monitor,
            resync_monitor,
            query_live_connections,
            get_connection,
            close_connection,
            get_settings,
            get_autostart_state,
            set_autostart_enabled,
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
            save_dimension_rank_table_layout,
            save_targets,
            test_controller,
            disconnect_controller,
            list_routes,
            pick_file,
            start_operation,
            cancel_operation,
            finish_operation,
            get_recovery_status,
            run_report,
            residential_share,
            list_report_archives,
            get_report_archive,
            get_report,
            release_report,
            preview_export,
            export_report,
            render_report_html,
            get_latest_residential_manual,
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
        .build(tauri::generate_context!())
        .expect("启动 ResiWatch 失败")
        .run(|app, event| {
            if matches!(
                event,
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
            ) {
                cancel_background_work(app);
                if let Some(state) = app.try_state::<Mutex<AppFacade>>() {
                    if let Ok(mut guard) = state.lock() {
                        if guard.desktop.shutdown == ShutdownPhase::Idle {
                            guard.shutdown();
                        }
                    }
                }
                stop_background_work(app);
            }
        });
}

#[cfg(test)]
mod background_worker_tests {
    use super::*;
    use std::cell::RefCell;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    #[test]
    fn busy_worker_keeps_one_signal_and_does_not_block_the_collector() {
        let cancel = Arc::new(AtomicBool::new(false));
        let (entered, observed) = mpsc::channel();
        let (release, waiting) = mpsc::channel();
        let mut count = 0;
        let worker = BackgroundWorker::spawn(cancel, move || {
            count += 1;
            entered.send(count).expect("worker entered");
            if count == 1 {
                waiting
                    .recv_timeout(Duration::from_secs(5))
                    .expect("release");
            }
            true
        });
        assert!(worker.signal());
        assert_eq!(
            observed
                .recv_timeout(Duration::from_secs(5))
                .expect("first"),
            1
        );
        let mut queued = 0;
        for _ in 0..100 {
            queued += usize::from(worker.signal());
        }
        assert_eq!(queued, 1);
        release.send(()).expect("unblock");
        assert_eq!(
            observed
                .recv_timeout(Duration::from_secs(5))
                .expect("one pending"),
            2
        );
        worker.stop();
        assert!(observed.try_recv().is_err());
    }

    #[test]
    fn background_shutdown_cancels_active_work_and_joins() {
        let cancel = Arc::new(AtomicBool::new(false));
        let running_cancel = Arc::clone(&cancel);
        let (entered, observed) = mpsc::channel();
        let (finished, exited) = mpsc::channel();
        let worker = BackgroundWorker::spawn(cancel, move || {
            entered.send(()).expect("entered");
            let deadline = Instant::now() + Duration::from_secs(5);
            while !running_cancel.load(Ordering::SeqCst) && Instant::now() < deadline {
                std::thread::yield_now();
            }
            finished
                .send(running_cancel.load(Ordering::SeqCst))
                .expect("finished");
            true
        });
        worker.signal();
        observed
            .recv_timeout(Duration::from_secs(5))
            .expect("running");
        worker.stop();
        assert!(exited.recv_timeout(Duration::from_secs(1)).expect("joined"));
    }

    #[test]
    fn exclusive_pause_joins_before_taking_the_facade_lock() {
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let facade = Arc::new(Mutex::new(false));
        let worker_facade = Arc::clone(&facade);
        let (entered, observed) = mpsc::channel();
        let mut owner = BackgroundWorkState {
            worker: Some(BackgroundWorker::spawn(Arc::clone(&cancel), move || {
                entered.send(()).expect("started");
                let deadline = Instant::now() + Duration::from_secs(5);
                while !worker_cancel.load(Ordering::SeqCst) && Instant::now() < deadline {
                    std::thread::yield_now();
                }
                *worker_facade.lock().expect("writer can finish") = true;
                true
            })),
            ..Default::default()
        };
        owner.worker.as_ref().expect("worker").signal();
        observed
            .recv_timeout(Duration::from_secs(5))
            .expect("started");
        owner.pause(&cancel);
        let guard = facade.lock().expect("exclusive facade");
        assert!(*guard, "old worker joined before exclusive database work");
        assert!(owner.worker.is_none());
        assert!(cancel.load(Ordering::SeqCst));
    }

    #[test]
    fn exclusive_failure_resumes_once_after_all_pauses_and_never_during_shutdown() {
        let cancel = AtomicBool::new(false);
        let mut owner = BackgroundWorkState::default();
        owner.pause(&cancel);
        owner.pause(&cancel);
        let failed_operation: Result<(), ()> = Err(());
        assert!(failed_operation.is_err());
        assert!(!owner.resume(&cancel, true));
        assert!(cancel.load(Ordering::SeqCst));
        assert!(owner.resume(&cancel, true));
        assert!(!cancel.load(Ordering::SeqCst));
        owner.pause(&cancel);
        owner.stopping = true;
        assert!(!owner.resume(&cancel, true));
        assert!(cancel.load(Ordering::SeqCst));
    }

    #[test]
    fn failed_archive_attempts_do_not_starve_maintenance() {
        let calls = RefCell::new(Vec::new());
        let mut prefer_maintenance = false;
        for _ in 0..6 {
            run_fair_background_step(
                &mut prefer_maintenance,
                || {
                    calls.borrow_mut().push("failed archive");
                    true
                },
                || {
                    calls.borrow_mut().push("maintenance");
                    true
                },
            );
        }
        assert_eq!(
            *calls.borrow(),
            [
                "failed archive",
                "maintenance",
                "failed archive",
                "maintenance",
                "failed archive",
                "maintenance"
            ]
        );
        calls.borrow_mut().clear();
        prefer_maintenance = true;
        run_fair_background_step(
            &mut prefer_maintenance,
            || {
                calls.borrow_mut().push("archive");
                true
            },
            || false,
        );
        assert_eq!(*calls.borrow(), ["archive"]);
    }
}

#[cfg(test)]
mod window_visibility_tests {
    use super::*;

    #[test]
    fn native_hidden_minimized_and_restored_states_publish_only_transitions() {
        let state = WindowVisibilityState::default();
        let mut events = Vec::new();
        // 后台启动、显示、焦点改变、最小化、恢复、关闭到托盘、再次显示。
        for (shown, minimized) in [
            (false, false),
            (true, false),
            (true, false),
            (true, true),
            (true, true),
            (true, false),
            (false, false),
            (false, true),
            (true, true),
            (true, false),
        ] {
            let value = visible_for_display(shown, minimized);
            if state.changed(value) {
                events.push(value.visible);
            }
        }
        assert_eq!(events, [false, true, false, true, false, true]);
        assert_eq!(
            serde_json::to_value(visible_for_display(true, false)).expect("dto"),
            serde_json::json!({"visible":true})
        );
    }
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
        assert!(!dir.path().join("archive-tick").exists());
    }

    #[test]
    fn archive_tick_respects_low_space_before_persisting() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.space = c3::SpaceBudget::exhausted();
        let state = Mutex::new(facade);
        archive_tick_at(&state, chrono::Utc::now().timestamp());
        assert_eq!(list_kind(&state, "hour"), 0);
        assert!(!dir.path().join("archive-tick").exists());
    }

    #[test]
    fn archive_tick_persist_failure_logs_class_without_secret() {
        let _lock = crate::app_log::exclusive_test();
        let dir = tempdir().expect("dir");
        let log_root = tempdir().expect("log root");
        let logs = log_root.path().to_path_buf();
        std::fs::create_dir_all(&logs).expect("logs dir");
        let _reset = crate::app_log::ResetOnDrop;
        crate::app_log::init_at(logs.clone(), crate::app_log::DEFAULT_MAX_BYTES);
        let facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        const FIXTURE_SECRET: &str = "password=fixture-secret-value";
        facade
            .storage
            .as_ref()
            .expect("storage")
            .connection()
            .execute_batch(&format!(
                "create trigger fail_archive_write before insert on report_archive begin
                   select raise(abort, '{FIXTURE_SECRET}');
                 end;"
            ))
            .expect("trigger");
        let state = Mutex::new(facade);
        assert!(archive_tick(&state));
        let text = crate::app_log::read_logged_text(&logs).expect("log");
        assert!(text.contains("ERROR archive_persist"), "{text}");
        assert!(text.contains("\"class\":\"storage_failure\""), "{text}");
        assert!(!text.contains("fixture-secret-value"), "{text}");
        assert!(!crate::redact::scan_text_for_secrets(&text), "{text}");
    }
}

#[cfg(test)]
mod autostart_command_tests {
    use super::*;
    use c2::desktop::FakeAutostart;
    use std::sync::Mutex;
    use tempfile::tempdir;

    const RAW_PLATFORM_ERROR: &str = "C:\\Users\\operator\\ResiWatch\\residential-monitor.exe; HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run; access denied";

    struct FixedReadbackPort {
        requested: Mutex<Option<bool>>,
        readback: bool,
    }

    impl AutostartPort for FixedReadbackPort {
        fn set_enabled(&self, enabled: bool) -> Result<(), AutostartError> {
            *self.requested.lock().expect("requested") = Some(enabled);
            Ok(())
        }

        fn is_enabled(&self) -> Result<bool, AutostartError> {
            Ok(self.readback)
        }
    }

    struct RawFailurePort;

    impl AutostartPort for RawFailurePort {
        fn set_enabled(&self, _enabled: bool) -> Result<(), AutostartError> {
            Err(AutostartError::unavailable(RAW_PLATFORM_ERROR))
        }

        fn is_enabled(&self) -> Result<bool, AutostartError> {
            Err(AutostartError::unavailable(RAW_PLATFORM_ERROR))
        }
    }

    #[test]
    fn reading_default_state_never_writes() {
        let port = FakeAutostart::new();
        let state = get_autostart_state_core(&port, UiLocale::Zh).expect("state");
        assert!(!state.enabled);
        assert_eq!(*port.requested.lock().expect("requested"), None);
    }

    #[test]
    fn set_command_returns_os_readback_instead_of_requested_value() {
        let port = FixedReadbackPort {
            requested: Mutex::new(None),
            readback: false,
        };
        let state = set_autostart_enabled_core(&port, UiLocale::Zh, true).expect("set");
        assert!(!state.enabled);
        assert_eq!(*port.requested.lock().expect("requested"), Some(true));
    }

    #[test]
    fn platform_detail_is_absent_from_ipc_and_log() {
        let _lock = crate::app_log::exclusive_test();
        let dir = tempdir().expect("log dir");
        let _reset = crate::app_log::ResetOnDrop;
        crate::app_log::init_at(dir.path().to_path_buf(), crate::app_log::DEFAULT_MAX_BYTES);

        let raw_error = AutostartError::unavailable(RAW_PLATFORM_ERROR);
        assert_eq!(raw_error.raw_detail(), RAW_PLATFORM_ERROR);
        assert!(!format!("{raw_error:?}").contains(RAW_PLATFORM_ERROR));

        let error =
            get_autostart_state_core(&RawFailurePort, UiLocale::En).expect_err("platform error");
        assert_eq!(error.code, "autostart_unavailable");
        assert_eq!(error.details_redacted, "autostart_unavailable");
        assert!(error.retryable);
        assert!(!error.message_zh.contains(RAW_PLATFORM_ERROR));
        assert!(!error.action.contains(RAW_PLATFORM_ERROR));
        assert!(!serde_json::to_string(&error)
            .expect("serialize")
            .contains(RAW_PLATFORM_ERROR));

        let log = crate::app_log::read_logged_text(dir.path()).expect("read log");
        assert!(log.contains("autostart"));
        assert!(log.contains("\"class\":\"platform\""));
        assert!(!log.contains(RAW_PLATFORM_ERROR));
        assert!(!log.contains("residential-monitor.exe"));
        assert!(!log.contains("CurrentVersion"));
    }
}

#[cfg(test)]
mod facade_lock_command_tests {
    use super::*;
    use c2::desktop::InstanceClaim;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use tempfile::tempdir;

    const FIXTURE_SECRET: &str = "poison-lock-secret-value";

    fn poison(state: &Mutex<AppFacade>) {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = state.lock().expect("lock");
            panic!("poison facade");
        }));
        assert!(state.lock().is_err());
    }

    #[test]
    fn poisoned_facade_command_returns_storage_failure_without_unwind() {
        let _lock = crate::app_log::exclusive_test();
        let dir = tempdir().expect("dir");
        let logs = dir.path().join("logs");
        let _reset = crate::app_log::ResetOnDrop;
        crate::app_log::init_at(logs.clone(), crate::app_log::DEFAULT_MAX_BYTES);
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade
            .save_controller(
                "127.0.0.1:9097".into(),
                Some(FIXTURE_SECRET.into()),
                true,
                true,
            )
            .expect("secret");
        let state = Mutex::new(facade);
        poison(&state);

        let caught = catch_unwind(AssertUnwindSafe(|| get_bootstrap_core(&state)));
        let error = caught
            .expect("command must not unwind")
            .expect_err("poison");
        assert_eq!(error.code, "storage_failure");
        assert_eq!(error.details_redacted, "storage_failure");
        assert!(error.retryable);
        let encoded = serde_json::to_string(&error).expect("dto");
        for text in [
            encoded.as_str(),
            error.message_zh.as_str(),
            error.details_redacted.as_str(),
            error.action.as_str(),
        ] {
            assert!(!text.contains(FIXTURE_SECRET), "{text}");
            assert!(!crate::redact::scan_text_for_secrets(text), "{text}");
        }

        let again = get_bootstrap_core(&state).expect_err("still poisoned");
        assert_eq!(again.code, "storage_failure");

        let log = crate::app_log::read_logged_text(&logs).expect("log");
        assert!(log.contains("facade_lock"), "{log}");
        assert!(log.contains("\"class\":\"mutex_poisoned\""), "{log}");
        assert!(!log.contains(FIXTURE_SECRET), "{log}");
        assert!(!crate::redact::scan_text_for_secrets(&log), "{log}");
    }
}

#[cfg(test)]
mod test_controller_command_tests {
    use super::*;
    use crate::identity::CREDENTIAL_TARGET;
    use crate::transport::spawn_fixture_server;
    use c2::desktop::InstanceClaim;
    use tempfile::tempdir;

    const OLD_SECRET: &str = "ac5-old-secret-token";
    const NEW_SECRET: &str = "ac5-new-secret-token";

    #[tokio::test]
    async fn failed_probe_leaves_stable_target_unchanged() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade
            .save_controller(
                "127.0.0.1:9097".into(),
                Some(OLD_SECRET.into()),
                false,
                true,
            )
            .expect("old");
        let (addr, stop) = spawn_fixture_server(Some(OLD_SECRET)).await;
        let state = Mutex::new(facade);
        let (result, messages) =
            test_controller_core(&state, addr.to_string(), Some(NEW_SECRET.into())).await;
        let error = result.expect_err("401");
        assert_eq!(error.code, "tcp_unauthorized");
        let guard = state.lock().expect("state");
        assert_eq!(
            guard.reveal_secret().expect("reveal").as_deref(),
            Some(OLD_SECRET)
        );
        assert!(guard
            .workflow
            .resolve(&format!("{CREDENTIAL_TARGET}/pending"), "persistent")
            .is_err());
        let encoded = serde_json::to_string(&error).expect("dto");
        let published = serde_json::to_string(&messages).expect("messages");
        for text in [
            encoded.as_str(),
            published.as_str(),
            error.message_zh.as_str(),
            error.details_redacted.as_str(),
        ] {
            assert!(!text.contains(OLD_SECRET), "{text}");
            assert!(!text.contains(NEW_SECRET), "{text}");
            assert!(!crate::redact::scan_text_for_secrets(text), "{text}");
        }
        let _ = stop.send(());
    }

    #[tokio::test]
    async fn successful_probe_promotes_secret() {
        let (addr, stop) = spawn_fixture_server(Some("echo-secret")).await;
        let dir = tempdir().expect("dir");
        let facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        let state = Mutex::new(facade);
        let (result, _) =
            test_controller_core(&state, addr.to_string(), Some("echo-secret".into())).await;
        result.expect("ok");
        let guard = state.lock().expect("state");
        assert_eq!(
            guard.reveal_secret().expect("reveal").as_deref(),
            Some("echo-secret")
        );
        let _ = stop.send(());
    }
}
