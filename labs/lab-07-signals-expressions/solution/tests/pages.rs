use topcoat::router::{Body, Method, StatusCode, request::Request, to_bytes};

async fn get(path: &str) -> (StatusCode, String, String) {
    let app_state = lab07_solution::auth::AppState::new();
    let router = lab07_solution::app::router(app_state.clone(), None);
    let request = Request::builder()
        .method(Method::GET)
        .uri(path)
        .body(Body::empty())
        .unwrap();
    let response = router.handle(request).await;
    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        content_type,
        String::from_utf8(body.to_vec()).unwrap(),
    )
}

#[tokio::test]
async fn module_pages_share_the_layout() {
    for path in ["/", "/berths", "/berths/a1"] {
        let (status, _, html) = get(path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
        assert!(html.contains("Slipway Marina"), "{html}");
    }
}

#[tokio::test]
async fn slug_path_parameter_renders_the_requested_berth() {
    let (status, _, html) = get("/berths/a1").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Berth A1"), "{html}");
    assert!(html.contains("Lady Jane"), "{html}");
    assert!(html.contains("data-berth=\"a1\""), "{html}");
}

#[tokio::test]
async fn unknown_slug_is_not_found() {
    let (status, _, _) = get("/berths/does-not-exist").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn health_route_returns_json() {
    let (status, content_type, body) = get("/api/health").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        content_type.starts_with("application/json"),
        "{content_type}"
    );
    assert_eq!(body, r#"{"status":"ok"}"#);
}

// ANCHOR: memoize-integration-test
#[tokio::test]
async fn memoized_berth_load_runs_once_for_three_components() {
    let app_state = lab07_solution::auth::AppState::new();
    let router = lab07_solution::app::router(app_state.clone(), None);
    let request = Request::builder()
        .method(Method::GET)
        .uri("/berths/a1")
        .body(Body::empty())
        .unwrap();

    let response = router.handle(request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("data-component=\"berth-title\""), "{html}");
    assert!(html.contains("data-component=\"berth-summary\""), "{html}");
    assert!(html.contains("data-component=\"berth-note\""), "{html}");
    assert_eq!(
        app_state
            .load_count
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}
// ANCHOR_END: memoize-integration-test
