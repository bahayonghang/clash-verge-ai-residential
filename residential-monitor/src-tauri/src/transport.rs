//! TCP / named pipe 兼容探测。HTTP 使用 hyper，不手写完整解析器。

use crate::c0_contract::FRAME_BODY_LIMIT;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::header::CONTENT_LENGTH;
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

/// connect / handshake / 读体总超时。超限映射为 `EndpointMissing`，采集循环可进入下一拍。
pub const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
/// 响应体上限。超过则 `ProtocolIncompatible`，不把整段读进 `String`。
pub const HTTP_BODY_MAX_BYTES: usize = FRAME_BODY_LIMIT;

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
) -> Result<(StatusCode, String), TransportErrorKind> {
    fetch_path(addr, "/version", secret).await
}

pub async fn fetch_connections(
    addr: SocketAddr,
    secret: Option<&str>,
) -> Result<(StatusCode, String), TransportErrorKind> {
    fetch_path(addr, "/connections", secret).await
}

pub async fn fetch_path(
    addr: SocketAddr,
    path: &str,
    secret: Option<&str>,
) -> Result<(StatusCode, String), TransportErrorKind> {
    fetch_path_method(addr, Method::GET, path, secret).await
}

pub async fn delete_connection(
    addr: SocketAddr,
    secret: Option<&str>,
    connection_id: &str,
) -> Result<StatusCode, TransportErrorKind> {
    if !connection_id_allowed(connection_id) {
        return Err(TransportErrorKind::ProtocolIncompatible);
    }
    let path = format!("/connections/{connection_id}");
    let (status, _) = fetch_path_method(addr, Method::DELETE, &path, secret).await?;
    Ok(status)
}

pub fn connection_id_allowed(connection_id: &str) -> bool {
    let len = connection_id.len();
    (1..=128).contains(&len)
        && connection_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

pub async fn fetch_path_method(
    addr: SocketAddr,
    method: Method,
    path: &str,
    secret: Option<&str>,
) -> Result<(StatusCode, String), TransportErrorKind> {
    match tokio::time::timeout(
        HTTP_REQUEST_TIMEOUT,
        fetch_path_method_inner(addr, method, path, secret),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(TransportErrorKind::EndpointMissing),
    }
}

async fn fetch_path_method_inner(
    addr: SocketAddr,
    method: Method,
    path: &str,
    secret: Option<&str>,
) -> Result<(StatusCode, String), TransportErrorKind> {
    let stream = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|_| TransportErrorKind::EndpointMissing)?;
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|_| TransportErrorKind::EndpointMissing)?;
    tokio::spawn(async move {
        let _ = conn.await;
    });
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "127.0.0.1");
    if let Some(secret) = secret {
        builder = builder.header("authorization", format!("Bearer {secret}"));
    }
    let request = builder
        .body(Full::new(Bytes::new()))
        .map_err(|_| TransportErrorKind::ProtocolIncompatible)?;
    let response = sender
        .send_request(request)
        .await
        .map_err(|_| TransportErrorKind::EndpointMissing)?;
    let status = response.status();
    if content_length_over_cap(response.headers()) {
        return Err(TransportErrorKind::ProtocolIncompatible);
    }
    let body = collect_body_limited(response.into_body(), HTTP_BODY_MAX_BYTES).await?;
    Ok((status, String::from_utf8_lossy(&body).into_owned()))
}

fn content_length_over_cap(headers: &hyper::HeaderMap) -> bool {
    headers
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|text| text.parse::<usize>().ok())
        .is_some_and(|len| len > HTTP_BODY_MAX_BYTES)
}

async fn collect_body_limited<B>(body: B, max_bytes: usize) -> Result<Vec<u8>, TransportErrorKind>
where
    B: hyper::body::Body,
    B::Data: AsRef<[u8]>,
{
    let mut body = std::pin::pin!(body);
    let mut collected = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(|_| TransportErrorKind::EndpointMissing)?;
        let Ok(data) = frame.into_data() else {
            continue;
        };
        let chunk = data.as_ref();
        if collected.len().saturating_add(chunk.len()) > max_bytes {
            return Err(TransportErrorKind::ProtocolIncompatible);
        }
        collected.extend_from_slice(chunk);
    }
    Ok(collected)
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
pub async fn spawn_oversize_connections_server() -> (SocketAddr, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind oversize");
    let addr = listener.local_addr().expect("local addr");
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                accepted = listener.accept() => {
                    let Ok((stream, _)) = accepted else { break; };
                    let io = TokioIo::new(stream);
                    tokio::spawn(async move {
                        let service = service_fn(move |request: Request<hyper::body::Incoming>| {
                            async move {
                                let body = if request.method() == Method::GET
                                    && request.uri().path() == "/connections"
                                {
                                    "x".repeat(HTTP_BODY_MAX_BYTES + 1)
                                } else {
                                    "{\"version\":\"c0-fixture\"}".into()
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
    (addr, shutdown_tx)
}

#[cfg(test)]
pub async fn spawn_stalling_body_server() -> (SocketAddr, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind stall");
    let addr = listener.local_addr().expect("local addr");
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
    tokio::spawn(async move {
        tokio::select! {
            _ = &mut shutdown_rx => {}
            accepted = listener.accept() => {
                let Ok((mut stream, _)) = accepted else { return; };
                use tokio::io::AsyncWriteExt;
                let _ = stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\n")
                    .await;
                let _ = shutdown_rx.await;
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;
    (addr, shutdown_tx)
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

    #[tokio::test]
    async fn fetch_oversize_body_is_protocol_incompatible() {
        let (addr, stop) = spawn_oversize_connections_server().await;
        let error = fetch_connections(addr, None).await.expect_err("oversize");
        assert_eq!(error, TransportErrorKind::ProtocolIncompatible);
        let _ = stop.send(());
    }

    #[tokio::test]
    async fn fetch_stalling_body_is_endpoint_missing() {
        let (addr, stop) = spawn_stalling_body_server().await;
        let started = std::time::Instant::now();
        let error = fetch_connections(addr, None).await.expect_err("timeout");
        assert_eq!(error, TransportErrorKind::EndpointMissing);
        assert!(started.elapsed() < HTTP_REQUEST_TIMEOUT + Duration::from_secs(1));
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
