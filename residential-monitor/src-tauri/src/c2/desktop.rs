//! 桌面生命周期状态机。不在测试中写本机自启动或安装态。

use crate::controller::ControllerInput;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LaunchMode {
    Interactive,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstanceClaim {
    Owner,
    FocusExisting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShutdownPhase {
    Idle,
    StopIntake,
    FlushWriter,
    CloseCoverage,
    Checkpoint,
    RemoveTray,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CloseWindowResult {
    Hidden,
    HiddenFirstExplain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraySummary {
    pub collector_running: bool,
    pub health: String,
    pub window_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopRuntime {
    pub launch_mode: LaunchMode,
    pub instance: InstanceClaim,
    pub window_visible: bool,
    pub close_explained: bool,
    pub collector_running: bool,
    pub shutdown: ShutdownPhase,
    pub focus_requested: bool,
}

impl DesktopRuntime {
    pub fn start(args: &[String], claim: InstanceClaim) -> Self {
        let background = args.iter().any(|item| item == "--background");
        Self {
            launch_mode: if background {
                LaunchMode::Background
            } else {
                LaunchMode::Interactive
            },
            instance: claim,
            window_visible: !background && claim == InstanceClaim::Owner,
            close_explained: false,
            collector_running: claim == InstanceClaim::Owner,
            shutdown: ShutdownPhase::Idle,
            focus_requested: claim == InstanceClaim::FocusExisting,
        }
    }

    pub fn close_window(&mut self) -> CloseWindowResult {
        self.window_visible = false;
        if self.close_explained {
            CloseWindowResult::Hidden
        } else {
            self.close_explained = true;
            CloseWindowResult::HiddenFirstExplain
        }
    }

    pub fn open_window(&mut self) {
        if self.instance == InstanceClaim::Owner && self.shutdown == ShutdownPhase::Idle {
            self.window_visible = true;
        }
    }

    pub fn request_focus_from_second_instance(&mut self) {
        self.focus_requested = true;
        self.open_window();
    }

    pub fn set_collector_running(&mut self, running: bool) -> ControllerInput {
        self.collector_running = running;
        if running {
            ControllerInput::Resumed
        } else {
            ControllerInput::Paused
        }
    }

    pub fn reconnect(&self) -> ControllerInput {
        ControllerInput::Disconnected {
            reason: crate::controller::SessionStatus::Cancelled,
        }
    }

    pub fn on_sleep(&self) -> ControllerInput {
        ControllerInput::SleepGap {
            started_utc: 0,
            ended_utc: 0,
        }
    }

    pub fn on_resume(&self) -> ControllerInput {
        ControllerInput::Resumed
    }

    pub fn begin_shutdown(&mut self) -> Vec<ShutdownPhase> {
        if self.shutdown != ShutdownPhase::Idle {
            return vec![self.shutdown];
        }
        let steps = [
            ShutdownPhase::StopIntake,
            ShutdownPhase::FlushWriter,
            ShutdownPhase::CloseCoverage,
            ShutdownPhase::Checkpoint,
            ShutdownPhase::RemoveTray,
            ShutdownPhase::Exit,
        ];
        self.shutdown = ShutdownPhase::Exit;
        self.collector_running = false;
        self.window_visible = false;
        steps.to_vec()
    }

    pub fn tray_summary(&self, health: &str) -> TraySummary {
        TraySummary {
            collector_running: self.collector_running,
            health: health.to_string(),
            window_visible: self.window_visible,
        }
    }
}

pub trait AutostartPort {
    fn set_enabled(&self, enabled: bool) -> Result<(), AutostartError>;
    fn is_enabled(&self) -> Result<bool, AutostartError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutostartError {
    Unavailable,
}

#[derive(Default)]
pub struct FakeAutostart {
    pub requested: std::sync::Mutex<Option<bool>>,
    pub os_state: std::sync::Mutex<bool>,
    pub fail_write: std::sync::Mutex<bool>,
}

impl FakeAutostart {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AutostartPort for FakeAutostart {
    fn set_enabled(&self, enabled: bool) -> Result<(), AutostartError> {
        *self.requested.lock().expect("autostart") = Some(enabled);
        if *self.fail_write.lock().expect("autostart") {
            return Err(AutostartError::Unavailable);
        }
        *self.os_state.lock().expect("autostart") = enabled;
        Ok(())
    }

    fn is_enabled(&self) -> Result<bool, AutostartError> {
        Ok(*self.os_state.lock().expect("autostart"))
    }
}

pub fn autostart_command_line(exe: &std::path::Path) -> String {
    format!("\"{}\" --background", exe.display())
}

pub struct ProcessSingleInstance {
    owner: bool,
}

impl ProcessSingleInstance {
    pub fn claim_first() -> Self {
        Self { owner: true }
    }

    pub fn claim_second() -> Self {
        Self { owner: false }
    }

    pub fn claim(&self) -> InstanceClaim {
        if self.owner {
            InstanceClaim::Owner
        } else {
            InstanceClaim::FocusExisting
        }
    }
}

#[cfg(test)]
mod desktop_lifecycle_tests {
    use super::*;

    #[test]
    fn second_instance_does_not_start_collector() {
        let first = DesktopRuntime::start(&["app".into()], InstanceClaim::Owner);
        let second = DesktopRuntime::start(&["app".into()], InstanceClaim::FocusExisting);
        assert!(first.collector_running);
        assert!(!second.collector_running);
        assert_eq!(second.instance, InstanceClaim::FocusExisting);
    }

    #[test]
    fn close_hides_and_background_stays_hidden() {
        let mut runtime = DesktopRuntime::start(&["app".into()], InstanceClaim::Owner);
        assert_eq!(
            runtime.close_window(),
            CloseWindowResult::HiddenFirstExplain
        );
        assert!(!runtime.window_visible);
        assert!(runtime.collector_running);
        assert_eq!(runtime.close_window(), CloseWindowResult::Hidden);
        let background =
            DesktopRuntime::start(&["app".into(), "--background".into()], InstanceClaim::Owner);
        assert!(!background.window_visible);
        assert_eq!(background.launch_mode, LaunchMode::Background);
    }

    #[test]
    fn shutdown_is_idempotent() {
        let mut runtime = DesktopRuntime::start(&["app".into()], InstanceClaim::Owner);
        let first = runtime.begin_shutdown();
        let second = runtime.begin_shutdown();
        assert_eq!(first.last().copied(), Some(ShutdownPhase::Exit));
        assert_eq!(second, vec![ShutdownPhase::Exit]);
        assert!(!runtime.collector_running);
    }

    #[test]
    fn pause_resume_and_sleep_map_to_c1_inputs() {
        let mut runtime = DesktopRuntime::start(&["app".into()], InstanceClaim::Owner);
        assert!(matches!(
            runtime.set_collector_running(false),
            ControllerInput::Paused
        ));
        assert!(matches!(
            runtime.set_collector_running(true),
            ControllerInput::Resumed
        ));
        assert!(matches!(
            runtime.on_sleep(),
            ControllerInput::SleepGap { .. }
        ));
        assert!(matches!(
            runtime.reconnect(),
            ControllerInput::Disconnected { .. }
        ));
    }

    #[test]
    fn autostart_command_uses_background_flag() {
        let line = autostart_command_line(std::path::Path::new("C:/app/residential-monitor.exe"));
        assert!(line.contains("--background"));
        assert!(line.contains("residential-monitor.exe"));
    }

    #[test]
    fn autostart_reads_os_state_not_request() {
        let port = FakeAutostart::new();
        *port.fail_write.lock().expect("f") = true;
        assert!(port.set_enabled(true).is_err());
        assert!(!port.is_enabled().expect("os"));
        *port.fail_write.lock().expect("f") = false;
        port.set_enabled(true).expect("write");
        assert!(port.is_enabled().expect("os"));
    }
}
