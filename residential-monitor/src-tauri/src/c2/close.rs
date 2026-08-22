//! 单条关闭请求。204 只表示已发送。

use crate::c2::contract::CLOSE_UNCONFIRMED_MS;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlResult {
    Accepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CloseMark {
    Accepted,
    Closed,
    Unconfirmed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseState {
    pub request_id: String,
    pub identity: String,
    pub mark: CloseMark,
}

struct Pending {
    request_id: String,
    started: Instant,
}

#[derive(Default)]
pub struct CloseRegistry {
    pending: HashMap<String, Pending>,
}

impl CloseRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accept(&mut self, identity: String, request_id: String) -> CloseState {
        self.pending.insert(
            identity.clone(),
            Pending {
                request_id: request_id.clone(),
                started: Instant::now(),
            },
        );
        CloseState {
            request_id,
            identity,
            mark: CloseMark::Accepted,
        }
    }

    pub fn mark_of(&self, identity: &str) -> Option<CloseMark> {
        self.pending.get(identity).map(|_| CloseMark::Accepted)
    }

    pub fn on_remove(&mut self, identity: &str) -> Option<CloseState> {
        self.pending.remove(identity).map(|pending| CloseState {
            request_id: pending.request_id,
            identity: identity.to_string(),
            mark: CloseMark::Closed,
        })
    }

    pub fn poll_timeouts(&mut self, now: Instant) -> Vec<CloseState> {
        let mut out = Vec::new();
        self.pending.retain(|identity, pending| {
            if now.duration_since(pending.started).as_millis() as u64 >= CLOSE_UNCONFIRMED_MS {
                out.push(CloseState {
                    request_id: pending.request_id.clone(),
                    identity: identity.clone(),
                    mark: CloseMark::Unconfirmed,
                });
                false
            } else {
                true
            }
        });
        out
    }
}

#[cfg(test)]
mod close_request_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn accepted_stays_until_remove() {
        let mut registry = CloseRegistry::new();
        let state = registry.accept("0:missing".into(), "req-1".into());
        assert_eq!(state.mark, CloseMark::Accepted);
        assert!(registry.on_remove("0:other").is_none());
        let closed = registry.on_remove("0:missing").expect("closed");
        assert_eq!(closed.mark, CloseMark::Closed);
    }

    #[test]
    fn timeout_is_unconfirmed() {
        let mut registry = CloseRegistry::new();
        registry.accept("0:a".into(), "req".into());
        let later = Instant::now() + Duration::from_millis(CLOSE_UNCONFIRMED_MS + 1);
        let timed = registry.poll_timeouts(later);
        assert_eq!(timed[0].mark, CloseMark::Unconfirmed);
    }
}
