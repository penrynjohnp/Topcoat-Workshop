use futures_util::{SinkExt, StreamExt};
use lab09_solution::auth::AppState;
use std::{net::SocketAddr, time::Duration};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};
use topcoat::{
    router::{Body, Method, StatusCode, request::Request, to_bytes},
    runtime::RUNTIME_PROTOCOL,
};

fn runtime_route(html: &str) -> (String, String) {
    let marker = "<!--::topcoat::shard::start(\"";
    let start = html.find(marker).expect("runtime endpoint marker") + marker.len();
    let rest = &html[start..];
    let shard_end = rest.find("\", \"").expect("shard endpoint path");
    let path = &rest[..shard_end];
    let identity_start = shard_end + 4;
    let identity_rest = &rest[identity_start..];
    let identity_end = identity_rest.find("\", [").expect("shard identity") + identity_start;
    let identity = &rest[identity_start..identity_end];
    (path.to_owned(), identity.to_owned())
}

fn signal_id(html: &str) -> String {
    let marker = "&quot;t&quot;:&quot;signal&quot;,&quot;id&quot;:&quot;";
    let start = html.find(marker).expect("signal marker") + marker.len();
    let end = html[start..].find("&quot;").expect("signal id") + start;
    html[start..end].to_owned()
}

fn procedure_path(html: &str) -> String {
    let marker = "&quot;t&quot;:&quot;Procedure&quot;,&quot;path&quot;:&quot;";
    let start = html.find(marker).expect("procedure marker") + marker.len();
    let end = html[start..]
        .find("&quot;")
        .expect("procedure endpoint path")
        + start;
    html[start..end].to_owned()
}

async fn response_body(response: topcoat::router::response::Response) -> String {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

// ANCHOR: shard-partial-response-test
#[tokio::test]
async fn vessel_shard_returns_only_the_filtered_fragment() {
    let router = lab09_solution::app::router(AppState::new(), None);
    let page = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri("/vessels")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let page = response_body(page).await;
    let (path, identity) = runtime_route(&page);
    let signal = signal_id(&page);

    let partial = router
        .handle(
            Request::builder()
                .method(Method::POST)
                .uri(path)
                .header("content-type", "application/json")
                .header("x-topcoat-identity", &identity)
                .body(Body::from(format!(
                    r#"{{"args":[{{"t":"Signal","id":"{signal}","v":"Lady"}}],"signals":{{}}}}"#
                )))
                .unwrap(),
        )
        .await;
    assert_eq!(partial.status(), StatusCode::OK);
    let html = response_body(partial).await;

    assert!(
        html.contains("<section data-component=\"vessel-results\" data-query=\"Lady\">"),
        "{html}"
    );
    assert!(html.contains("id=\"vessel-lady-jane\""), "{html}");
    assert!(html.contains("Lady Jane"), "{html}");
    assert!(!html.contains("Sea Urchin"), "{html}");
    assert!(!html.contains("Morning Star"), "{html}");
    assert!(!html.contains("<!DOCTYPE html>"), "{html}");
    assert!(!html.contains("<html"), "{html}");
    assert!(!html.contains("<body"), "{html}");
    assert!(!html.contains("Slipway Marina"), "{html}");
    assert!(!html.contains("<main>"), "{html}");
    assert!(!html.contains("id=\"vessel-query\""), "{html}");
}
// ANCHOR_END: shard-partial-response-test

async fn login(router: &topcoat::router::Router) -> String {
    let request = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri("/login/request?email=runtime@example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let body = response_body(request).await;
    let token = body
        .split("token=")
        .nth(1)
        .and_then(|value| value.split('"').next())
        .unwrap();
    let verify = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/login/verify?token={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    verify
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn procedure_persists_completion_and_history_streams_errors_in_place() {
    let state = AppState::new();
    let router = lab09_solution::app::router(state.clone(), None);
    let cookie = login(&router).await;

    let dashboard = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri("/dashboard")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let dashboard = response_body(dashboard).await;
    let procedure = procedure_path(&dashboard);

    let unauthorized = router
        .handle(
            Request::builder()
                .method(Method::POST)
                .uri(&procedure)
                .header("content-type", "application/json")
                .body(Body::from(r#"["wo-a1-antifoul"]"#))
                .unwrap(),
        )
        .await;
    assert_eq!(unauthorized.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(unauthorized.headers().get("location").unwrap(), "/login");

    let saved = router
        .handle(
            Request::builder()
                .method(Method::POST)
                .uri(procedure)
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"["wo-a1-antifoul"]"#))
                .unwrap(),
        )
        .await;
    assert_eq!(saved.status(), StatusCode::OK);
    assert_eq!(response_body(saved).await, "true");
    assert!(state.work_order_is_complete("wo-a1-antifoul"));

    let history = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri("/dashboard/history")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let history = response_body(history).await;
    assert!(history.contains("Loading history"), "{history}");
    assert!(history.contains("work-order-history"), "{history}");
    assert!(history.contains("data-topcoat-swap="), "{history}");

    let failing = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri("/dashboard/history?fail=true")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let failing = response_body(failing).await;
    assert!(failing.contains("History is unavailable"), "{failing}");
    assert!(failing.contains("history store unavailable"), "{failing}");
    assert!(failing.contains("Work-order history"), "{failing}");

    std::fs::remove_dir_all("mail").ok();
}

type RuntimeSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn open_runtime_socket(
    address: SocketAddr,
    path: &str,
    identity: &str,
    cookie: &str,
    run: u64,
) -> RuntimeSocket {
    let mut request = format!("ws://{address}{path}")
        .into_client_request()
        .unwrap();
    request
        .headers_mut()
        .insert("cookie", cookie.parse().unwrap());
    request
        .headers_mut()
        .insert("sec-websocket-protocol", RUNTIME_PROTOCOL.parse().unwrap());
    let (mut socket, response) = connect_async(request).await.unwrap();
    assert_eq!(
        response.headers()["sec-websocket-protocol"],
        RUNTIME_PROTOCOL
    );
    socket
        .send(Message::Text(
            serde_json::json!({ "run": run, "shard": identity, "args": [], "signals": {} })
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    socket
}

async fn receive_until(socket: &mut RuntimeSocket, expected: &str) -> String {
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut received = String::new();
        loop {
            let message = socket.next().await.unwrap().unwrap();
            if let Message::Text(text) = message {
                assert!(!text.contains("\"t\":\"error\""), "{text}");
                received.push_str(&text);
                if received.contains(expected) {
                    return received;
                }
            }
        }
    })
    .await
    .expect("runtime frame did not arrive")
}

async fn wait_for_subscribers(state: &AppState, expected: usize) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while state.updates.receiver_count() != expected {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("live render did not release its subscription");
}

// ANCHOR: connected-runtime-test
#[tokio::test]
async fn connected_status_updates_reconnects_and_rechecks_auth() {
    let state = AppState::new();
    let router = lab09_solution::app::router(state.clone(), None);
    let cookie = login(&router).await;
    let history = router
        .handle(
            Request::builder()
                .uri("/dashboard/history")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let history = tokio::time::timeout(Duration::from_secs(5), response_body(history))
        .await
        .expect("initial HTTP render did not finish");
    let (path, identity) = runtime_route(&history);
    assert_eq!(path, "/dashboard/work-orders/status");
    assert!(history.contains("Completed work orders: 0"), "{history}");
    assert_eq!(state.updates.receiver_count(), 0);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (shutdown, stopped) = tokio::sync::oneshot::channel();
    let server_router = router.clone();
    let server = tokio::spawn(async move {
        topcoat::serve_until(listener, server_router, async {
            stopped.await.ok();
        })
        .await
        .unwrap();
    });

    let mut socket = open_runtime_socket(address, &path, &identity, &cookie, 1).await;
    receive_until(&mut socket, "Completed work orders: 0").await;
    wait_for_subscribers(&state, 1).await;

    let dashboard = router
        .handle(
            Request::builder()
                .uri("/dashboard")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let procedure = procedure_path(&response_body(dashboard).await);
    let saved = router
        .handle(
            Request::builder()
                .method(Method::POST)
                .uri(procedure)
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!(["wo-a1-antifoul"]).to_string(),
                ))
                .unwrap(),
        )
        .await;
    assert_eq!(saved.status(), StatusCode::OK);
    receive_until(&mut socket, "Completed work orders: 1").await;

    socket.close(None).await.unwrap();
    wait_for_subscribers(&state, 0).await;
    let mut reconnected = open_runtime_socket(address, &path, &identity, &cookie, 2).await;
    receive_until(&mut reconnected, "Completed work orders: 1").await;

    let logout = router
        .handle(
            Request::builder()
                .method(Method::POST)
                .uri("/logout")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert!(logout.status().is_redirection());
    let expired = receive_until(&mut reconnected, "Your session has expired.").await;
    assert!(!expired.contains("Completed work orders"), "{expired}");
    wait_for_subscribers(&state, 0).await;
    reconnected.close(None).await.unwrap();

    shutdown.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .unwrap()
        .unwrap();
    std::fs::remove_dir_all("mail").ok();
}
// ANCHOR_END: connected-runtime-test
