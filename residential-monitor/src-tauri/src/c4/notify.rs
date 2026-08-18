//! Windows 通知 seam。默认不发送真实系统通知。

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

#[derive(Debug, Default)]
pub struct WindowsNotificationSink {
    pub allow_real: bool,
}

impl WindowsNotificationSink {
    pub fn from_env() -> Self {
        Self {
            allow_real: std::env::var("RESIDENTIAL_MONITOR_ALLOW_TOAST")
                .ok()
                .as_deref()
                == Some("1"),
        }
    }
}

impl NotificationSink for WindowsNotificationSink {
    fn capability(&self) -> NotifyCapability {
        if !cfg!(windows) {
            return NotifyCapability {
                available: false,
                reason_zh: "v1 只在 Windows 11 提供系统通知。".into(),
                can_focus_app: false,
                focus_assist_unknown: true,
            };
        }
        if !self.allow_real {
            return NotifyCapability {
                available: false,
                reason_zh: "未授权发送系统通知。应用内告警中心仍是权威记录。".into(),
                can_focus_app: true,
                focus_assist_unknown: true,
            };
        }
        NotifyCapability {
            available: true,
            reason_zh: "将尝试提交 Windows 通知。Focus Assist 或系统关闭时用户可能看不到。".into(),
            can_focus_app: true,
            focus_assist_unknown: true,
        }
    }

    fn send(&mut self, payload: &NotifyPayload) -> Result<(), NotifyError> {
        let _ = payload;
        if !self.allow_real || !cfg!(windows) {
            return Err(NotifyError::Disabled("not authorized"));
        }
        Err(NotifyError::Disabled("real toast requires reconfirmation"))
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
        let win = WindowsNotificationSink { allow_real: false };
        assert!(!win.capability().available);
        assert!(win.capability().reason_zh.contains("未授权"));
    }
}
