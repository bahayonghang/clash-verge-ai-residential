//! 订阅表：保存 Channel/sink，不进入 AppFacade。

use crate::c2::hub::MonitorStreamMessage;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SinkSendError;

pub trait MonitorSink {
    fn send_monitor(&self, message: MonitorStreamMessage) -> Result<(), SinkSendError>;
}

impl MonitorSink for tauri::ipc::Channel<MonitorStreamMessage> {
    fn send_monitor(&self, message: MonitorStreamMessage) -> Result<(), SinkSendError> {
        self.send(message).map_err(|_| SinkSendError)
    }
}

#[derive(Default)]
pub struct SubscriptionRegistry<S> {
    sinks: BTreeMap<u64, S>,
}

impl<S> SubscriptionRegistry<S> {
    pub fn new() -> Self {
        Self {
            sinks: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, id: u64, sink: S) {
        self.sinks.insert(id, sink);
    }

    pub fn remove(&mut self, id: u64) -> Option<S> {
        self.sinks.remove(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.sinks.is_empty()
    }
}

impl<S: MonitorSink> SubscriptionRegistry<S> {
    /// 转发一条消息。发送失败的订阅会被移除。
    pub fn forward(&mut self, message: &MonitorStreamMessage) -> Vec<u64> {
        if self.sinks.is_empty() {
            return Vec::new();
        }
        let mut dead = Vec::new();
        for (id, sink) in &self.sinks {
            if sink.send_monitor(message.clone()).is_err() {
                dead.push(*id);
            }
        }
        for id in &dead {
            self.sinks.remove(id);
        }
        dead
    }
}

#[cfg(test)]
mod subscription_forward_tests {
    use super::*;
    use crate::accounting::AccountingEngine;
    use crate::c2::hub::{health_from, LiveConnectionView, MonitorHub};
    use crate::controller::{ControllerInput, SessionStatus};
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct MockSink {
        received: Arc<Mutex<Vec<MonitorStreamMessage>>>,
        fail: bool,
    }

    impl MonitorSink for MockSink {
        fn send_monitor(&self, message: MonitorStreamMessage) -> Result<(), SinkSendError> {
            if self.fail {
                return Err(SinkSendError);
            }
            self.received.lock().expect("sink").push(message);
            Ok(())
        }
    }

    fn row(id: &str) -> LiveConnectionView {
        LiveConnectionView {
            identity: format!("0:{id}"),
            connection_id: id.into(),
            epoch: 0,
            upload: 1,
            download: 1,
            rate_upload: None,
            rate_download: None,
            duration_ms: None,
            primary: None,
            tags: Vec::new(),
            host: Some(format!("{id}.test")),
            source_ip: None,
            destination_ip: None,
            process_name: None,
            process_path: None,
            network: Some("tcp".into()),
            rule: None,
            rule_payload: None,
            chains: Vec::new(),
        }
    }

    #[test]
    fn after_subscribe_publish_reaches_stored_sink() {
        let hub = MonitorHub::new();
        let bootstrap = hub.subscribe();
        let MonitorStreamMessage::Bootstrap {
            subscription_id, ..
        } = bootstrap
        else {
            panic!("bootstrap");
        };
        let received = Arc::new(Mutex::new(Vec::new()));
        let mut registry = SubscriptionRegistry::new();
        registry.insert(
            subscription_id,
            MockSink {
                received: received.clone(),
                fail: false,
            },
        );
        registry.forward(&bootstrap);
        let batch = AccountingEngine::new().apply(ControllerInput::Paused, 1, 1);
        let published = hub
            .publish(
                &batch,
                vec![row("live")],
                health_from(SessionStatus::Connected, None),
                1,
            )
            .expect("publish")
            .expect("delta");
        assert!(matches!(
            published,
            MonitorStreamMessage::ConnectionDelta { .. }
        ));
        registry.forward(&published);
        let messages = received.lock().expect("sink");
        assert!(matches!(
            messages.first(),
            Some(MonitorStreamMessage::Bootstrap { .. })
        ));
        assert!(matches!(
            messages.get(1),
            Some(MonitorStreamMessage::ConnectionDelta { .. })
        ));
    }

    #[test]
    fn no_subscribers_does_not_serialize_delta() {
        let hub = MonitorHub::new();
        let batch = AccountingEngine::new().apply(ControllerInput::Paused, 1, 1);
        let published = hub
            .publish(
                &batch,
                vec![row("gone")],
                health_from(SessionStatus::Connected, None),
                1,
            )
            .expect("publish");
        assert!(published.is_none());
        assert_eq!(hub.row_count(), 1);
    }

    #[test]
    fn failed_send_drops_subscription() {
        let mut registry = SubscriptionRegistry::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        registry.insert(
            7,
            MockSink {
                received,
                fail: true,
            },
        );
        let dead = registry.forward(&MonitorStreamMessage::HealthChanged {
            schema_version: 1,
            subscription_id: 7,
            seq: 1,
            health: health_from(SessionStatus::Connected, None),
            backend_time: 1,
        });
        assert_eq!(dead, vec![7]);
        assert!(registry.is_empty());
    }
}
