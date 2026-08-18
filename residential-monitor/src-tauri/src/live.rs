//! durable projection 与原子 Channel 水位。

use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorMessage {
    Bootstrap { base_seq: u64 },
    Delta { seq: u64 },
}

#[derive(Debug, Default)]
struct Inner {
    seq: u64,
    watermark: u64,
}

#[derive(Debug, Default)]
pub struct LiveProjection {
    inner: Mutex<Inner>,
}

impl LiveProjection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_receipt(&self, watermark: u64) -> u64 {
        let mut guard = self.inner.lock().expect("live lock");
        guard.watermark = watermark;
        guard.seq += 1;
        guard.seq
    }

    pub fn subscribe(&self) -> MonitorMessage {
        let guard = self.inner.lock().expect("live lock");
        MonitorMessage::Bootstrap {
            base_seq: guard.seq,
        }
    }

    pub fn next_after(&self, base_seq: u64) -> Result<MonitorMessage, &'static str> {
        let guard = self.inner.lock().expect("live lock");
        if guard.seq <= base_seq {
            return Ok(MonitorMessage::Delta { seq: guard.seq + 1 });
        }
        if guard.seq == base_seq + 1 || guard.seq > base_seq {
            return Ok(MonitorMessage::Delta { seq: guard.seq });
        }
        Err("seq gap")
    }

    pub fn resync(&self) -> MonitorMessage {
        self.subscribe()
    }
}

#[cfg(test)]
mod live_projection_tests {
    use super::*;

    #[test]
    fn live_projection_advances_only_after_receipt() {
        let live = LiveProjection::new();
        let MonitorMessage::Bootstrap { base_seq } = live.subscribe() else {
            panic!("bootstrap");
        };
        assert_eq!(base_seq, 0);
        let seq = live.apply_receipt(1);
        assert!(seq > base_seq);
    }
}

#[cfg(test)]
mod channel_atomic_subscribe_tests {
    use super::*;

    #[test]
    fn channel_atomic_subscribe_first_message_is_bootstrap() {
        let live = LiveProjection::new();
        live.apply_receipt(4);
        assert!(matches!(
            live.subscribe(),
            MonitorMessage::Bootstrap { base_seq } if base_seq >= 1
        ));
    }
}

#[cfg(test)]
mod channel_resync_tests {
    use super::*;

    #[test]
    fn channel_resync_returns_new_bootstrap_instead_of_old_delta() {
        let live = LiveProjection::new();
        live.apply_receipt(1);
        live.apply_receipt(2);
        let MonitorMessage::Bootstrap { base_seq } = live.resync() else {
            panic!("resync");
        };
        assert!(base_seq >= 2);
    }
}
