use lab07_solution::auth::AppState;
use topcoat::router::{Body, Method, StatusCode, request::Request, to_bytes};

async fn get(path: &str) -> (StatusCode, String) {
    let router = lab07_solution::app::router(AppState::new(), None);
    let response = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri(path)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

// ANCHOR: reactivity-integration-test
#[tokio::test]
async fn berth_filter_ships_signals_handlers_and_binds() {
    let (status, html) = get("/berths").await;
    assert_eq!(status, StatusCode::OK);

    // Three signals are serialized into the page: query, show_occupied, show_vacant.
    assert_eq!(html.matches("::topcoat::signal(").count(), 3, "{html}");
    // The input writes into the query signal and reads it back.
    assert!(html.contains("data-topcoat-on:input="), "{html}");
    assert!(html.contains("data-topcoat-bind:value="), "{html}");
    // Both status chips toggle a bool signal and bind their class.
    assert_eq!(html.matches("data-topcoat-on:click=").count(), 2, "{html}");
    assert_eq!(
        html.matches("data-topcoat-bind:class=").count(),
        2,
        "{html}"
    );
    // Every row carries the hidden expression.
    assert_eq!(
        html.matches("data-topcoat-bind:hidden=").count(),
        4,
        "{html}"
    );
}

#[tokio::test]
async fn every_berth_is_visible_on_first_render() {
    let (_, html) = get("/berths").await;
    for slug in ["a1", "a2", "b7", "c3"] {
        assert!(html.contains(&format!("data-berth=\"{slug}\"")), "{html}");
    }
    // The server evaluated each expression to `false`, so no row starts hidden.
    assert!(!html.contains("hidden=\"\""), "{html}");
}

#[tokio::test]
async fn tracked_server_read_emits_one_dependency_marker() {
    let (status, html) = get("/").await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(html.matches("::topcoat::signal(").count(), 2, "{html}");
    assert_eq!(html.matches("::topcoat::dep(").count(), 1, "{html}");
    assert!(html.contains("data-server-read=\"tracked\""), "{html}");
    assert!(html.contains("data-server-read=\"untracked\""), "{html}");
}

#[tokio::test]
async fn client_only_filter_needs_no_shard_or_procedure_endpoint() {
    let (_, html) = get("/berths").await;
    assert!(!html.contains("/_topcoat/shard"), "{html}");
    assert!(!html.contains("/_topcoat/procedure"), "{html}");
}
// ANCHOR_END: reactivity-integration-test

#[tokio::test]
async fn runtime_script_is_only_rendered_with_a_loaded_bundle() {
    let (_, html) = get("/berths").await;
    assert!(!html.contains("/_topcoat/assets/"), "{html}");
}

#[tokio::test]
async fn work_order_panel_starts_collapsed_for_an_authenticated_user() {
    let router = lab07_solution::app::router(AppState::new(), None);
    let response = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri("/login/request?email=ada@example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
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
    let cookie = verify
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();

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
    assert_eq!(dashboard.status(), StatusCode::OK);
    let html = to_bytes(dashboard.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(html.to_vec()).unwrap();

    assert!(html.contains("data-toggle=\"work-orders\""), "{html}");
    assert!(html.contains("data-topcoat-bind:hidden="), "{html}");
    assert!(html.contains("data-topcoat-bind:aria-expanded="), "{html}");
    // The panel is collapsed on the server render, so its body ships hidden.
    assert!(
        html.contains("data-panel=\"work-orders\" hidden=\"\""),
        "{html}"
    );

    std::fs::remove_dir_all("mail").ok();
}
