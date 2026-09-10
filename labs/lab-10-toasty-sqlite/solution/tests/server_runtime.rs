use lab10_solution::{auth::AppState, models::WorkOrder};
use topcoat::router::{Body, Method, StatusCode, request::Request, to_bytes};

fn runtime_path(html: &str, prefix: &str) -> String {
    let start = html.find(prefix).expect("runtime endpoint marker");
    html[start..start + prefix.len() + 36].to_owned()
}

fn procedure_path(html: &str) -> String {
    let marker = "&quot;t&quot;:&quot;Procedure&quot;,&quot;id&quot;:&quot;";
    let start = html.find(marker).expect("procedure marker") + marker.len();
    format!("/_topcoat/procedures/{}", &html[start..start + 36])
}

async fn response_body(response: topcoat::router::response::Response) -> String {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

// ANCHOR: shard-partial-response-test
#[tokio::test]
async fn vessel_shard_returns_only_the_filtered_fragment() {
    let router = lab10_solution::app::router(AppState::test().await.unwrap(), None);
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
    let path = runtime_path(&page, "/_topcoat/shards/");

    let partial = router
        .handle(
            Request::builder()
                .method(Method::POST)
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(r#"["Lady"]"#))
                .unwrap(),
        )
        .await;
    assert_eq!(partial.status(), StatusCode::OK);
    let html = response_body(partial).await;

    assert!(html.contains("Lady Jane"), "{html}");
    assert!(!html.contains("Sea Urchin"), "{html}");
    assert!(!html.contains("Morning Star"), "{html}");
    assert!(!html.contains("<!DOCTYPE html>"), "{html}");
    assert!(!html.contains("Slipway Marina"), "{html}");
    assert!(!html.contains("<main>"), "{html}");
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
    let state = AppState::test().await.unwrap();
    let router = lab10_solution::app::router(state.clone(), None);
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
                .body(Body::from(r#"[1.0]"#))
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
                .body(Body::from(r#"[1.0]"#))
                .unwrap(),
        )
        .await;
    assert_eq!(saved.status(), StatusCode::OK);
    assert_eq!(response_body(saved).await, "true");
    let mut db = state.db.clone();
    let saved_order = WorkOrder::get_by_id(&mut db, &1).await.unwrap();
    assert!(saved_order.completed);

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
