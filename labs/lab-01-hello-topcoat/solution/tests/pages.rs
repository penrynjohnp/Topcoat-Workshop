//! Integration test: build the real router, request the page, assert on HTML.

use topcoat::router::{
    Body, Method, Router, RouterBuilderDiscoverExt, StatusCode, request::Request, to_bytes,
};

#[tokio::test]
async fn home_page_greets_topcoat() {
    assert!(!lab01_solution::NAME.is_empty());

    let router = Router::builder().discover().build();

    let request = Request::builder()
        .method(Method::GET)
        .uri("/")
        .body(Body::empty())
        .expect("valid request");

    let response = router.handle(request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let html = String::from_utf8(body.to_vec()).expect("utf-8");

    assert!(html.contains("<h1>Hello, Topcoat!</h1>"), "{html}");
}
