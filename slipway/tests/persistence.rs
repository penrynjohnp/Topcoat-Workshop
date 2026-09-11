use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use slipway_capstone::{
    auth::AppState,
    database,
    models::{Berth, SessionRecord, WorkOrder},
};
use topcoat::{
    cookie::Key,
    router::{Body, Method, Router, StatusCode, request::Request, to_bytes},
};

async fn body(response: topcoat::router::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

async fn login(router: &Router, email: &str) -> String {
    let response = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/login/request?email={email}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let html = body(response).await;
    let token = html
        .split("token=")
        .nth(1)
        .and_then(|value| value.split('"').next())
        .unwrap();
    let response = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/login/verify?token={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    response
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

// ANCHOR: relation-query-test
#[tokio::test]
async fn seeded_relations_are_queryable() {
    let state = AppState::test().await.unwrap();
    let mut db = state.db.clone();
    let berth = Berth::filter_by_slug("a1")
        .include(Berth::fields().vessels())
        .include(Berth::fields().work_orders())
        .first()
        .exec(&mut db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(berth.vessels.get().len(), 1);
    assert_eq!(berth.vessels.get()[0].name, "Lady Jane");
    assert_eq!(berth.work_orders.get().len(), 1);
    assert_eq!(berth.work_orders.get()[0].title, "Anti-foul A1");
}
// ANCHOR_END: relation-query-test

// ANCHOR: form-validation-test
#[tokio::test]
async fn invalid_form_rerenders_values_and_does_not_insert() {
    let state = AppState::test().await.unwrap();
    let router = slipway_capstone::app::router(state.clone(), None);
    let cookie = login(&router, "form-errors@example.com").await;
    let title = "x".repeat(81);

    let response = router
        .handle(
            Request::builder()
                .method(Method::POST)
                .uri("/dashboard/work-orders/new")
                .header("cookie", cookie)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(format!("title={title}&berth_slug=yard")))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let html = body(response).await;
    assert!(html.contains(&format!("value=\"{title}\"")), "{html}");
    assert!(
        html.contains("Keep the title to 80 characters or fewer."),
        "{html}"
    );
    assert!(html.contains("Choose a managed berth."), "{html}");
    assert!(
        html.contains(r#"value="yard" selected="selected""#),
        "{html}"
    );

    let mut db = state.db.clone();
    let orders = WorkOrder::all().exec(&mut db).await.unwrap();
    assert_eq!(orders.len(), 2);
}
// ANCHOR_END: form-validation-test

#[tokio::test]
async fn valid_form_creates_through_the_berth_relation() {
    let state = AppState::test().await.unwrap();
    let router = slipway_capstone::app::router(state.clone(), None);
    let cookie = login(&router, "form-success@example.com").await;

    let response = router
        .handle(
            Request::builder()
                .method(Method::POST)
                .uri("/dashboard/work-orders/new")
                .header("cookie", cookie)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("title=Inspect+pedestal&berth_slug=a1"))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/dashboard");

    let mut db = state.db.clone();
    let order = WorkOrder::all()
        .exec(&mut db)
        .await
        .unwrap()
        .into_iter()
        .find(|order| order.title == "Inspect pedestal")
        .unwrap();
    assert_eq!(order.berth_slug, "a1");
}

// ANCHOR: restart-persistence-test
#[tokio::test]
async fn data_and_sessions_survive_reopening_a_file_database() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("slipway-{suffix}.db"));
    let url = format!("sqlite:{}", path.display());
    let cookie_key = Arc::new(Key::generate());

    let db = database::connect(&url).await.unwrap();
    let state = AppState::from_db_with_cookie_key(db, cookie_key.clone());
    let router = slipway_capstone::app::router(state.clone(), None);
    let cookie = login(&router, "restart@example.com").await;
    let mut first_db = state.db.clone();
    let a1 = Berth::get_by_slug(&mut first_db, "a1").await.unwrap();
    toasty::create!(in a1.work_orders() {
        title: "Persistent inspection",
        completed: false,
    })
    .exec(&mut first_db)
    .await
    .unwrap();
    drop(first_db);
    drop(router);
    drop(state);

    let reopened = database::connect(&url).await.unwrap();
    let reopened_state = AppState::from_db_with_cookie_key(reopened, cookie_key);
    let mut reopened_db = reopened_state.db.clone();
    let orders = WorkOrder::all().exec(&mut reopened_db).await.unwrap();
    assert!(
        orders
            .iter()
            .any(|order| order.title == "Persistent inspection")
    );
    assert_eq!(
        SessionRecord::all()
            .exec(&mut reopened_db)
            .await
            .unwrap()
            .len(),
        1
    );

    let router = slipway_capstone::app::router(reopened_state, None);
    let response = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri("/admin")
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(body(response).await.contains("restart@example.com"));

    drop(reopened_db);
    drop(router);
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_file(format!("{}-shm", path.display())).ok();
    std::fs::remove_file(format!("{}-wal", path.display())).ok();
    std::fs::remove_dir_all("mail").ok();
}
// ANCHOR_END: restart-persistence-test
