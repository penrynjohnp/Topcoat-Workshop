use slipway::auth::AppState;
use topcoat::router::{Body, Method, Router, StatusCode, request::Request, response::Response};
use topcoat::router::{header, to_bytes};

async fn body_of(response: Response) -> String {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

async fn get(router: &Router, uri: &str) -> Response {
    router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
}

async fn hx_get(router: &Router, uri: &str) -> Response {
    router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .header("hx-request", "true")
                .body(Body::empty())
                .unwrap(),
        )
        .await
}

fn header_value(response: &Response, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .map(|value| value.to_str().unwrap().to_owned())
}

fn shard_path(html: &str) -> String {
    let prefix = "/_topcoat/shards/";
    let start = html.find(prefix).expect("shard endpoint marker");
    html[start..start + prefix.len() + 36].to_owned()
}

// ANCHOR: coexistence-test
#[tokio::test]
async fn both_implementations_coexist_without_borrowing_each_other_s_mechanism() {
    let router = slipway::app::router(AppState::test().await.unwrap(), None);

    let native = body_of(get(&router, "/vessels").await).await;
    let htmx = body_of(get(&router, "/vessels/htmx").await).await;

    // The native page drives its search through a shard endpoint and ships no
    // htmx attributes of its own.
    assert!(native.contains("/_topcoat/shards/"), "{native}");
    assert!(!native.contains("hx-get"), "{native}");

    // The htmx page drives its search through attributes and registers no
    // reactive scope for the search.
    assert!(htmx.contains(r#"hx-get="/vessels/htmx/results""#), "{htmx}");
    assert!(!htmx.contains("/_topcoat/shards/"), "{htmx}");

    // Both are reachable from the app's navigation.
    assert!(native.contains(r#"href="/vessels/htmx""#), "{native}");
}
// ANCHOR_END: coexistence-test

// ANCHOR: fragment-equality-test
#[tokio::test]
async fn the_shard_and_the_htmx_route_render_the_same_fragment() {
    let router = slipway::app::router(AppState::test().await.unwrap(), None);

    let page = body_of(get(&router, "/vessels").await).await;
    let shard = router
        .handle(
            Request::builder()
                .method(Method::POST)
                .uri(shard_path(&page))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"["Lady"]"#))
                .unwrap(),
        )
        .await;
    let shard = body_of(shard).await;

    let fragment = body_of(hx_get(&router, "/vessels/htmx/results?q=Lady").await).await;

    // The transport differs; the markup does not. Both call `vessel_matches`.
    assert_eq!(shard, fragment);
    assert!(fragment.contains("Lady Jane"), "{fragment}");
}
// ANCHOR_END: fragment-equality-test

#[tokio::test]
async fn the_fragment_carries_only_matches_and_no_document_shell() {
    let router = slipway::app::router(AppState::test().await.unwrap(), None);
    let response = hx_get(&router, "/vessels/htmx/results?q=Lady").await;

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_of(response).await;

    assert!(html.contains("Lady Jane"), "{html}");
    assert!(!html.contains("Sea Urchin"), "{html}");
    assert!(!html.contains("<!DOCTYPE html>"), "{html}");
    assert!(!html.contains("Slipway Marina"), "{html}");
    assert!(!html.contains("<main>"), "{html}");
}

// ANCHOR: hx-request-guard-test
#[tokio::test]
async fn a_human_opening_the_fragment_url_is_sent_to_the_page() {
    let router = slipway::app::router(AppState::test().await.unwrap(), None);
    // No `HX-Request` header: this is a browser address bar, not htmx.
    let response = get(&router, "/vessels/htmx/results?q=Lady").await;

    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        header_value(&response, "location").as_deref(),
        Some("/vessels/htmx")
    );
}
// ANCHOR_END: hx-request-guard-test

// ANCHOR: response-header-test
#[tokio::test]
async fn a_successful_search_pushes_the_url_and_triggers_an_event() {
    let router = slipway::app::router(AppState::test().await.unwrap(), None);
    let response = hx_get(&router, "/vessels/htmx/results?q=Lady").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        header_value(&response, "hx-push-url").as_deref(),
        Some("/vessels/htmx?q=Lady")
    );
    // The count rides along as JSON event data.
    let trigger = header_value(&response, "hx-trigger-after-swap").expect("trigger header");
    assert!(trigger.contains("vessels:searched"), "{trigger}");
    assert!(trigger.contains('1'), "{trigger}");
}

#[tokio::test]
async fn an_over_long_query_is_retargeted_into_the_alert_region() {
    let router = slipway::app::router(AppState::test().await.unwrap(), None);
    let query = "a".repeat(200);
    let response = hx_get(&router, &format!("/vessels/htmx/results?q={query}")).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        header_value(&response, "hx-retarget").as_deref(),
        Some("#vessel-alert")
    );
    assert_eq!(
        header_value(&response, "hx-reswap").as_deref(),
        Some("innerHTML")
    );
    assert!(header_value(&response, "hx-push-url").is_none());

    let html = body_of(response).await;
    assert!(html.contains(r#"data-vessel-error="too-long""#), "{html}");
    assert!(!html.contains("Lady Jane"), "{html}");
}
// ANCHOR_END: response-header-test

// ANCHOR: no-js-test
#[tokio::test]
async fn the_page_searches_without_javascript() {
    let router = slipway::app::router(AppState::test().await.unwrap(), None);
    // What a plain form submission — or a shared link — produces.
    let html = body_of(get(&router, "/vessels/htmx?q=Lady").await).await;

    assert!(
        html.contains(r#"<form method="get" action="/vessels/htmx""#),
        "{html}"
    );
    assert!(html.contains("Lady Jane"), "{html}");
    assert!(!html.contains("Sea Urchin"), "{html}");
}
// ANCHOR_END: no-js-test
