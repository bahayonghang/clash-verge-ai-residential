//! 可单测采集节拍：短锁读配置，HTTP 期间不持 facade 锁。

use crate::c2::desktop::ShutdownPhase;
use crate::c2::facade::{parse_socket, AppFacade};
use crate::c2::hub::MonitorStreamMessage;
use crate::c2::shell::BootBranch;
use crate::controller::{reject_non_loopback_ip, ControllerInput, SessionStatus};
use crate::session::ControllerSession;
use std::net::SocketAddr;

/// 一次节拍的取帧计划。secret 只在栈上，不写日志或 Channel。
pub struct CollectorPlan {
    pub should_fetch: bool,
    address: Option<SocketAddr>,
    secret: Option<String>,
}

impl CollectorPlan {
    fn idle() -> Self {
        Self {
            should_fetch: false,
            address: None,
            secret: None,
        }
    }

    pub fn address(&self) -> Option<SocketAddr> {
        self.address
    }

    pub fn secret(&self) -> Option<&str> {
        self.secret.as_deref()
    }
}

/// 读快照：恢复模式、暂停、断开、空地址或非回环时跳过 HTTP。
pub fn plan_tick(facade: &AppFacade) -> CollectorPlan {
    if facade.branch != BootBranch::NormalReady {
        return CollectorPlan::idle();
    }
    if !facade.desktop.collector_running {
        return CollectorPlan::idle();
    }
    if facade.desktop.shutdown != ShutdownPhase::Idle {
        return CollectorPlan::idle();
    }
    if matches!(facade.session_status, SessionStatus::Cancelled) {
        return CollectorPlan::idle();
    }
    if facade.settings.address.is_empty() {
        return CollectorPlan::idle();
    }
    let Ok(addr) = parse_socket(&facade.settings.address) else {
        return CollectorPlan::idle();
    };
    if reject_non_loopback_ip(addr.ip()).is_err() {
        return CollectorPlan::idle();
    }
    let secret = if facade.settings.has_secret {
        facade
            .workflow
            .resolve(
                &facade.settings.credential_target,
                &facade.settings.secret_mode,
            )
            .ok()
            .map(|value| String::from_utf8_lossy(value.as_header_bytes()).into_owned())
    } else {
        None
    };
    CollectorPlan {
        should_fetch: true,
        address: Some(addr),
        secret,
    }
}

pub async fn fetch_snapshot(
    addr: SocketAddr,
    secret: Option<&str>,
) -> Result<ControllerInput, SessionStatus> {
    ControllerSession::fetch_normalized_snapshot(addr, secret).await
}

pub fn apply_tick_result(
    facade: &mut AppFacade,
    result: Result<ControllerInput, SessionStatus>,
) -> Option<MonitorStreamMessage> {
    match result {
        Ok(input) => {
            let utc = chrono::Utc::now().timestamp();
            facade.ingest_snapshot(input, utc, utc as u64)
        }
        Err(status) => facade.apply_probe_err(status),
    }
}

#[cfg(test)]
mod collector_tick_tests {
    use super::*;
    use crate::c2::desktop::InstanceClaim;
    use http_body_util::Full;
    use hyper::body::Bytes;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Method, Request, Response, StatusCode};
    use hyper_util::rt::TokioIo;
    use std::convert::Infallible;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::net::TcpListener;
    use tokio::sync::oneshot;

    async fn spawn_scripted_connections(
        bodies: Vec<&'static str>,
    ) -> (SocketAddr, Arc<AtomicUsize>, oneshot::Sender<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind collector fixture");
        let addr = listener.local_addr().expect("addr");
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_accept = hits.clone();
        let bodies = Arc::new(bodies);
        let (stop_tx, mut stop_rx) = oneshot::channel();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    accepted = listener.accept() => {
                        let Ok((stream, _)) = accepted else { break; };
                        let io = TokioIo::new(stream);
                        let hits = hits_accept.clone();
                        let bodies = bodies.clone();
                        tokio::spawn(async move {
                            let service = service_fn(move |request: Request<hyper::body::Incoming>| {
                                let hits = hits.clone();
                                let bodies = bodies.clone();
                                async move {
                                    let body = if request.method() == Method::GET
                                        && request.uri().path() == "/connections"
                                    {
                                        let index = hits.fetch_add(1, Ordering::SeqCst);
                                        bodies
                                            .get(index)
                                            .copied()
                                            .or_else(|| bodies.last().copied())
                                            .unwrap_or("{}")
                                    } else {
                                        "{\"downloadTotal\":0,\"uploadTotal\":0,\"connections\":[]}"
                                    };
                                    let response = Response::builder()
                                        .status(StatusCode::OK)
                                        .header("content-type", "application/json")
                                        .body(Full::new(Bytes::from(body)))
                                        .expect("response");
                                    Ok::<_, Infallible>(response)
                                }
                            });
                            let _ = http1::Builder::new().serve_connection(io, service).await;
                        });
                    }
                }
            }
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        (addr, hits, stop_tx)
    }

    fn boot_with_addr(addr: SocketAddr) -> (tempfile::TempDir, AppFacade) {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.settings.address = addr.to_string();
        (dir, facade)
    }

    #[test]
    fn empty_address_skips_fetch() {
        let dir = tempdir().expect("dir");
        let facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        let plan = plan_tick(&facade);
        assert!(!plan.should_fetch);
        assert!(plan.address().is_none());
    }

    #[tokio::test]
    async fn two_ticks_change_hub_rows() {
        let (addr, hits, stop) = spawn_scripted_connections(vec![
            "{\"downloadTotal\":1,\"uploadTotal\":1,\"connections\":[{\"id\":\"a\",\"upload\":1,\"download\":1,\"metadata\":{\"host\":\"a.test\",\"network\":\"tcp\"}}]}",
            "{\"downloadTotal\":2,\"uploadTotal\":2,\"connections\":[{\"id\":\"b\",\"upload\":2,\"download\":2,\"metadata\":{\"host\":\"b.test\",\"network\":\"tcp\"}}]}",
        ])
        .await;
        let (_dir, mut facade) = boot_with_addr(addr);
        let plan = plan_tick(&facade);
        assert!(plan.should_fetch);
        let first = fetch_snapshot(plan.address().expect("addr"), plan.secret())
            .await
            .expect("tick1");
        apply_tick_result(&mut facade, Ok(first));
        let first_ids: Vec<String> = facade
            .hub
            .rows()
            .into_iter()
            .map(|row| row.connection_id)
            .collect();
        assert_eq!(first_ids, vec!["a".to_string()]);
        let second = fetch_snapshot(addr, None).await.expect("tick2");
        apply_tick_result(&mut facade, Ok(second));
        let second_ids: Vec<String> = facade
            .hub
            .rows()
            .into_iter()
            .map(|row| row.connection_id)
            .collect();
        assert_eq!(second_ids, vec!["b".to_string()]);
        assert_ne!(first_ids, second_ids);
        assert_eq!(hits.load(Ordering::SeqCst), 2);
        let _ = stop.send(());
    }

    #[tokio::test]
    async fn pause_stops_requests_and_keeps_rows() {
        let (addr, hits, stop) = spawn_scripted_connections(vec![
            "{\"downloadTotal\":1,\"uploadTotal\":1,\"connections\":[{\"id\":\"keep\",\"upload\":1,\"download\":1,\"metadata\":{\"host\":\"keep.test\",\"network\":\"tcp\"}}]}",
        ])
        .await;
        let (_dir, mut facade) = boot_with_addr(addr);
        let plan = plan_tick(&facade);
        let snapshot = fetch_snapshot(plan.address().expect("addr"), plan.secret())
            .await
            .expect("tick");
        apply_tick_result(&mut facade, Ok(snapshot));
        assert_eq!(facade.hub.row_count(), 1);
        let before = hits.load(Ordering::SeqCst);
        let _ = facade.desktop.set_collector_running(false);
        let _ = facade.apply_lifecycle(ControllerInput::Paused);
        assert_eq!(facade.hub.row_count(), 1);
        assert_eq!(facade.hub.rows()[0].connection_id, "keep");
        let paused = plan_tick(&facade);
        assert!(!paused.should_fetch);
        assert_eq!(hits.load(Ordering::SeqCst), before);
        let _ = stop.send(());
    }

    #[test]
    fn disconnect_skips_fetch_reconnect_allows_next_tick() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.settings.address = "127.0.0.1:9097".into();
        assert!(plan_tick(&facade).should_fetch);
        facade.disconnect_now();
        assert_eq!(facade.session_status, SessionStatus::Cancelled);
        assert!(!plan_tick(&facade).should_fetch);
        facade.reconnect_now();
        assert_eq!(facade.session_status, SessionStatus::Connecting);
        assert!(plan_tick(&facade).should_fetch);
    }

    #[test]
    fn resume_after_disconnect_allows_next_tick() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.settings.address = "127.0.0.1:9097".into();
        facade.disconnect_now();
        let _ = facade.desktop.set_collector_running(false);
        assert!(!plan_tick(&facade).should_fetch);
        facade.resume_collector();
        assert_eq!(facade.session_status, SessionStatus::Connecting);
        assert!(plan_tick(&facade).should_fetch);
    }
}
