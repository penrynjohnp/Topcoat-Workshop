// ANCHOR: tower-bridge-tests
use lab13_solution::build_router;
use topcoat::router::{Body, Method, StatusCode, request::Request, to_bytes};

#[tokio::test]
async fn axum_health_route_is_mounted_under_api_v1() {
    let router = build_router();
    let request = Request::builder()
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = router.handle(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(payload, "{\"status\":\"ok\"}");
}

#[tokio::test]
async fn bridge_preserves_methods_and_topcoat_pages() {
    let router = build_router();
    for (method, uri, expected) in [
        (Method::GET, "/api/v1/health?probe=true", StatusCode::OK),
        (
            Method::POST,
            "/api/v1/health",
            StatusCode::METHOD_NOT_ALLOWED,
        ),
        (Method::GET, "/api/v1/missing", StatusCode::NOT_FOUND),
        (Method::GET, "/", StatusCode::OK),
    ] {
        let response = router
            .handle(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;
        assert_eq!(response.status(), expected, "{uri}");
    }
}
// ANCHOR_END: tower-bridge-tests
