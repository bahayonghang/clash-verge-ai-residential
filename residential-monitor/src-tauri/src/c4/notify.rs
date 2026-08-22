//! Windows 通知 seam。默认发送真实系统通知，环境变量可关闭。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyPayload {
    pub title_zh: String,
    pub body_zh: String,
    pub event_id: String,
    pub instance_id: Option<String>,
    pub test_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyCapability {
    pub available: bool,
    pub reason_zh: String,
    pub can_focus_app: bool,
    pub focus_assist_unknown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotifyError {
    Temporary(&'static str),
    Permanent(&'static str),
    Disabled(&'static str),
}

impl NotifyError {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Temporary(_) => "temporary",
            Self::Permanent(_) => "permanent",
            Self::Disabled(_) => "disabled",
        }
    }

    pub fn summary_zh(&self) -> &'static str {
        match self {
            Self::Temporary(_) => "通知发送暂时失败，将重试。",
            Self::Permanent(_) => "通知被系统拒绝，已标记失败。",
            Self::Disabled(_) => "系统通知不可用或未授权发送。",
        }
    }

    pub fn permanent(&self) -> bool {
        matches!(self, Self::Permanent(_) | Self::Disabled(_))
    }
}

pub trait NotificationSink {
    fn capability(&self) -> NotifyCapability;
    fn send(&mut self, payload: &NotifyPayload) -> Result<(), NotifyError>;
    /// 桌面运行时建立后补装 AppHandle。测试替身空实现。
    fn attach(&mut self, _app: tauri::AppHandle) {}
}

#[derive(Debug, Default)]
pub struct FakeNotificationSink {
    pub sent: Vec<NotifyPayload>,
    pub fail: Option<NotifyError>,
    pub available: bool,
}

impl NotificationSink for FakeNotificationSink {
    fn capability(&self) -> NotifyCapability {
        NotifyCapability {
            available: self.available,
            reason_zh: if self.available {
                "测试通知 seam 可用。".into()
            } else {
                "当前为进程内 FakeNotificationSink，未发送系统通知。".into()
            },
            can_focus_app: true,
            focus_assist_unknown: true,
        }
    }

    fn send(&mut self, payload: &NotifyPayload) -> Result<(), NotifyError> {
        if let Some(error) = &self.fail {
            return Err(error.clone());
        }
        self.sent.push(payload.clone());
        Ok(())
    }
}

pub struct WindowsNotificationSink {
    app: Option<tauri::AppHandle>,
    disabled: bool,
}

impl WindowsNotificationSink {
    /// `RESIDENTIAL_MONITOR_ALLOW_TOAST` 默认发送；值为 `0` 或 `false` 时关闭。
    pub fn new() -> Self {
        let disabled = std::env::var("RESIDENTIAL_MONITOR_ALLOW_TOAST")
            .ok()
            .map(|raw| matches!(raw.trim(), "0" | "false"))
            .unwrap_or(false);
        Self {
            app: None,
            disabled,
        }
    }
}

impl Default for WindowsNotificationSink {
    fn default() -> Self {
        Self::new()
    }
}

/// sink 的四种状态。`capability()` 与 `send()` 都经由 `state()` 派发，
/// 不存在 `available == true` 而 `send()` 命中 `Disabled` 的状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SinkState {
    /// 非 Windows。v1 只在 Windows 11 提供系统通知。
    NotWindows,
    /// `RESIDENTIAL_MONITOR_ALLOW_TOAST` 为 `0` / `false`。
    EnvDisabled,
    /// 桌面运行时尚未 attach。`boot()` 先于 Tauri 应用建立。
    NotReady,
    Ready,
}

impl WindowsNotificationSink {
    fn state(&self) -> SinkState {
        if !cfg!(windows) {
            SinkState::NotWindows
        } else if self.disabled {
            SinkState::EnvDisabled
        } else if self.app.is_none() {
            SinkState::NotReady
        } else {
            SinkState::Ready
        }
    }
}

fn capability_for(state: SinkState) -> NotifyCapability {
    match state {
        SinkState::NotWindows => NotifyCapability {
            available: false,
            reason_zh: "v1 只在 Windows 11 提供系统通知。".into(),
            can_focus_app: false,
            focus_assist_unknown: true,
        },
        SinkState::EnvDisabled => NotifyCapability {
            available: false,
            reason_zh: "系统通知已由环境变量 RESIDENTIAL_MONITOR_ALLOW_TOAST 关闭。".into(),
            can_focus_app: true,
            focus_assist_unknown: true,
        },
        SinkState::NotReady => NotifyCapability {
            available: false,
            reason_zh: "桌面运行时尚未就绪，暂不能发送系统通知。".into(),
            can_focus_app: true,
            focus_assist_unknown: true,
        },
        SinkState::Ready => NotifyCapability {
            available: true,
            reason_zh: "将尝试提交 Windows 通知。Focus Assist 或系统关闭时用户可能看不到。".into(),
            can_focus_app: true,
            focus_assist_unknown: true,
        },
    }
}

/// `send()` 在进入插件前的否决分支；`None` 即放行（只有 `Ready`）。
fn early_disabled(state: SinkState) -> Option<NotifyError> {
    match state {
        SinkState::NotWindows => Some(NotifyError::Disabled("platform")),
        SinkState::EnvDisabled => Some(NotifyError::Disabled("turned off")),
        SinkState::NotReady => Some(NotifyError::Disabled("runtime not ready")),
        SinkState::Ready => None,
    }
}

impl NotificationSink for WindowsNotificationSink {
    fn capability(&self) -> NotifyCapability {
        capability_for(self.state())
    }

    fn attach(&mut self, app: tauri::AppHandle) {
        self.app = Some(app);
    }

    fn send(&mut self, payload: &NotifyPayload) -> Result<(), NotifyError> {
        if let Some(error) = early_disabled(self.state()) {
            return Err(error);
        }
        let Some(app) = self.app.clone() else {
            return Err(NotifyError::Disabled("runtime not ready"));
        };
        use tauri_plugin_notification::NotificationExt;
        app.notification()
            .builder()
            .title(&payload.title_zh)
            .body(&payload.body_zh)
            .show()
            .map_err(|_| NotifyError::Temporary("show failed"))
    }
}

#[cfg(test)]
mod notify_seam_tests {
    use super::*;

    #[test]
    fn test_and_real_share_trait() {
        let mut fake = FakeNotificationSink {
            available: true,
            ..FakeNotificationSink::default()
        };
        fake.send(&NotifyPayload {
            title_zh: "测试通知".into(),
            body_zh: "不会写入告警历史。".into(),
            event_id: "test".into(),
            instance_id: None,
            test_only: true,
        })
        .expect("send");
        assert_eq!(fake.sent.len(), 1);
        assert!(fake.sent[0].test_only);
        let win = WindowsNotificationSink::new();
        assert!(!win.capability().available);
        assert!(win.capability().reason_zh.contains("尚未就绪"));
    }

    fn probe() -> NotifyPayload {
        NotifyPayload {
            title_zh: "t".into(),
            body_zh: "b".into(),
            event_id: "e".into(),
            instance_id: None,
            test_only: false,
        }
    }

    #[test]
    fn capability_and_send_agree_on_every_state() {
        // 不存在 available == true 且 send() 命中 Disabled 的状态。
        for state in [
            SinkState::NotWindows,
            SinkState::EnvDisabled,
            SinkState::NotReady,
            SinkState::Ready,
        ] {
            assert_eq!(
                capability_for(state).available,
                early_disabled(state).is_none(),
                "{state:?}: capability 与 send 的可用判定不一致"
            );
        }
    }

    #[test]
    fn unattached_sink_is_unavailable_and_disabled() {
        let mut win = WindowsNotificationSink::new();
        assert!(!win.capability().available);
        assert!(matches!(win.send(&probe()), Err(NotifyError::Disabled(_))));
    }
}
