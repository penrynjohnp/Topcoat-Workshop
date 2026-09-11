use slipway_capstone::auth::AppState;
use topcoat::router::{Body, Method, StatusCode, request::Request, to_bytes};

#[tokio::test]
async fn axum_health_route_is_mounted_under_api_v1() {
    let router = slipway_capstone::app::router(AppState::test().await.unwrap(), None);
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = router.handle(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), br#"{"status":"ok"}"#);
}
