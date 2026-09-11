use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use lab13_solution::api_router;

#[tokio::test]
async fn axum_health_route_is_mounted_under_api_v1() {
    let router = Router::new().nest("/api/v1", api_router());
    let request = Request::builder()
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = ServiceExt::oneshot(router, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let payload = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(payload, "{\"status\":\"ok\"}");
}
