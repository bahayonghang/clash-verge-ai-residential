//! TCP / named pipe 兼容探测。HTTP 使用 hyper，不手写完整解析器。

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileStatus {
    Supported,
    BestEffort,
    Incompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerProfile {
    pub name: &'static str,
    pub transport: &'static str,
    pub status: ProfileStatus,
    pub sends_secret: bool,
    pub tcp_fallback: bool,
    pub notes_zh: &'static str,
}

pub fn profiles() -> Vec<ControllerProfile> {
    vec![
        ControllerProfile {
            name: "tcp-loopback-secret",
            transport: "tcp",
            status: ProfileStatus::Supported,
            sends_secret: true,
            tcp_fallback: false,
            notes_zh: "TCP External Controller 是受支持路径，secret 使用 Bearer。",
        },
        ControllerProfile {
            name: "verge-2.5.2-fixed-pipe",
            transport: "named-pipe",
            status: ProfileStatus::BestEffort,
            sends_secret: false,
            tcp_fallback: true,
            notes_zh: "v2.5.2 固定管道 verge-mihomo；不发送 secret。",
        },
        ControllerProfile {
            name: "verge-dynamic-sidecar-service",
            transport: "named-pipe",
            status: ProfileStatus::BestEffort,
            sends_secret: false,
            tcp_fallback: true,
            notes_zh: "动态 sidecar/service 管道名按运行模式派生，必须先验身份。",
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportErrorKind {
    AuthFailed,
    PipeAccessDenied,
    PipeBusyTimeout,
    EndpointMissing,
    ProtocolIncompatible,
    PidMismatch,
    Cancelled,
    NonLoopback,
}

pub fn map_os_error(code: i32) -> TransportErrorKind {
    match code {
        2 => TransportErrorKind::EndpointMissing,
        5 => TransportErrorKind::PipeAccessDenied,
        231 => TransportErrorKind::PipeBusyTimeout,
        121 => TransportErrorKind::PipeBusyTimeout,
        _ => TransportErrorKind::ProtocolIncompatible,
    }
}

pub fn reject_non_loopback(host: &str) -> Result<(), TransportErrorKind> {
    if matches!(host, "127.0.0.1" | "::1" | "localhost") {
        Ok(())
    } else {
        Err(TransportErrorKind::NonLoopback)
    }
}

pub async fn fetch_version(
    addr: SocketAddr,
    secret: Option<&str>,
) -> Result<(StatusCode, String), String> {
    fetch_path(addr, "/version", secret).await
}

pub async fn fetch_connections(
    addr: SocketAddr,
    secret: Option<&str>,
) -> Result<(StatusCode, String), String> {
    fetch_path(addr, "/connections", secret).await
}

pub async fn fetch_path(
    addr: SocketAddr,
    path: &str,
    secret: Option<&str>,
) -> Result<(StatusCode, String), String> {
    let stream = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|error| error.to_string())?;
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|error| error.to_string())?;
    tokio::spawn(async move {
        let _ = conn.await;
    });
    let mut builder = Request::builder()
        .method(Method::GET)
        .uri(path)
        .header("host", "127.0.0.1");
    if let Some(secret) = secret {
        builder = builder.header("authorization", format!("Bearer {secret}"));
    }
    let request = builder
        .body(Full::new(Bytes::new()))
        .map_err(|error| error.to_string())?;
    let response = sender
        .send_request(request)
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|error| error.to_string())?
        .to_bytes();
    Ok((status, String::from_utf8_lossy(&body).into_owned()))
}

pub async fn spawn_fixture_server(
    expected_secret: Option<&'static str>,
) -> (SocketAddr, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fixture");
    let addr = listener.local_addr().expect("local addr");
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                accepted = listener.accept() => {
                    let Ok((stream, _)) = accepted else { break; };
                    let io = TokioIo::new(stream);
                    let expected = expected_secret;
                    tokio::spawn(async move {
                        let service = service_fn(move |request: Request<hyper::body::Incoming>| {
                            async move { Ok::<_, Infallible>(handle_fixture(request, expected).await) }
                        });
                        let _ = http1::Builder::new().serve_connection(io, service).await;
                    });
                }
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;
    (addr, shutdown_tx)
}

async fn handle_fixture(
    request: Request<hyper::body::Incoming>,
    expected_secret: Option<&str>,
) -> Response<Full<Bytes>> {
    if let Some(expected) = expected_secret {
        let header = request
            .headers()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        if header != format!("Bearer {expected}") {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Full::new(Bytes::from_static(
                    b"{\"message\":\"unauthorized\"}",
                )))
                .expect("response");
        }
    }
    match (request.method(), request.uri().path()) {
        (&Method::GET, "/version") => json(StatusCode::OK, "{\"version\":\"c0-fixture\"}"),
        (&Method::GET, "/connections") => json(
            StatusCode::OK,
            "{\"downloadTotal\":0,\"uploadTotal\":0,\"connections\":[]}",
        ),
        (&Method::GET, "/proxies") => {
            let bulky = format!(
                "{{\"proxies\":{}}}",
                "[\"".to_string() + &"x".repeat(64) + "\"]"
            );
            json(StatusCode::OK, &bulky)
        }
        (&Method::DELETE, path) if path.starts_with("/connections/") => Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Full::new(Bytes::new()))
            .expect("response"),
        _ => json(StatusCode::NOT_FOUND, "{\"message\":\"missing\"}"),
    }
}

fn json(status: StatusCode, body: &str) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(body.to_string())))
        .expect("response")
}

#[cfg(test)]
mod transport_fixture_tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    #[tokio::test]
    async fn transport_fixture_websocket_sends_one_frame() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind ws");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut ws = tokio_tungstenite::accept_async(stream)
                .await
                .expect("accept ws");
            ws.send(Message::Text(
                "{\"downloadTotal\":0,\"uploadTotal\":0,\"connections\":[]}".into(),
            ))
            .await
            .expect("send");
        });
        let url = format!("ws://{addr}/connections?interval=1000");
        let (mut client, _) = tokio_tungstenite::connect_async(url).await.expect("client");
        let frame = client.next().await.expect("frame").expect("ok");
        assert!(frame.to_text().unwrap_or("").contains("connections"));
    }

    #[tokio::test]
    async fn transport_fixture_tcp_secret_states() {
        reject_non_loopback("127.0.0.1").expect("loopback");
        assert_eq!(
            reject_non_loopback("8.8.8.8").unwrap_err(),
            TransportErrorKind::NonLoopback
        );

        let (addr, stop) = spawn_fixture_server(Some("fixture-secret")).await;
        let (ok, body) = fetch_version(addr, Some("fixture-secret"))
            .await
            .expect("ok");
        assert_eq!(ok, StatusCode::OK);
        assert!(body.contains("c0-fixture"));
        let (denied, _) = fetch_version(addr, Some("wrong")).await.expect("denied");
        assert_eq!(denied, StatusCode::UNAUTHORIZED);
        let (missing, _) = fetch_version(addr, None).await.expect("missing");
        assert_eq!(missing, StatusCode::UNAUTHORIZED);
        let _ = stop.send(());
    }
}

#[cfg(test)]
mod named_pipe_faults_tests {
    use super::*;

    #[test]
    fn named_pipe_faults_map_win32_codes() {
        assert_eq!(map_os_error(5), TransportErrorKind::PipeAccessDenied);
        assert_eq!(map_os_error(231), TransportErrorKind::PipeBusyTimeout);
        assert_eq!(map_os_error(2), TransportErrorKind::EndpointMissing);
    }
}

#[cfg(test)]
mod controller_profiles_tests {
    use super::*;

    #[test]
    fn controller_profiles_mark_tcp_supported_and_pipe_best_effort() {
        let all = profiles();
        let tcp = all
            .iter()
            .find(|item| item.name == "tcp-loopback-secret")
            .unwrap();
        assert_eq!(tcp.status, ProfileStatus::Supported);
        assert!(tcp.sends_secret);
        assert!(all
            .iter()
            .all(|item| item.transport != "named-pipe" || !item.sends_secret));
        assert!(all
            .iter()
            .filter(|item| item.transport == "named-pipe")
            .all(|item| item.tcp_fallback));
    }
}
