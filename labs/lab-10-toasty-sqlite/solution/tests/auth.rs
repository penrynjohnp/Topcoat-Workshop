use std::{fs, path::Path};

use lab10_solution::auth::AppState;
use topcoat::router::{Body, Method, StatusCode, request::Request, to_bytes};

async fn request(
    method: Method,
    path: &str,
    cookie: Option<&str>,
) -> (StatusCode, String, String, Option<String>) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(cookie) = cookie {
        builder = builder.header("cookie", cookie);
    }
    let response = lab10_solution::app::router(AppState::test().await.unwrap(), None)
        .handle(builder.body(Body::empty()).unwrap())
        .await;
    let status = response.status();
    let location = response
        .headers()
        .get("location")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let set_cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        String::from_utf8(body.to_vec()).unwrap(),
        set_cookie.unwrap_or_default(),
        location,
    )
}

// ANCHOR: auth-integration-test
#[tokio::test]
async fn unauthenticated_admin_redirects_to_login() {
    let (status, _, _, location) = request(Method::GET, "/admin", None).await;
    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(location.as_deref(), Some("/login"));
}

#[tokio::test]
async fn protected_component_embedded_on_public_dashboard_redirects() {
    let (status, _, _, location) = request(Method::GET, "/dashboard", None).await;
    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(location.as_deref(), Some("/login"));
}

#[tokio::test]
async fn magic_link_logs_in_and_logout_invalidates_the_session() {
    let mail_dir = Path::new("mail");
    if mail_dir.exists() {
        fs::remove_dir_all(mail_dir).unwrap();
    }

    let state = AppState::test().await.unwrap();
    let router = lab10_solution::app::router(state, None);
    let response = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri("/login/request?email=ada@example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    let token = body
        .split("token=")
        .nth(1)
        .and_then(|value| value.split('"').next())
        .unwrap();
    let mail_file = fs::read_dir(mail_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let mail = fs::read_to_string(mail_file).unwrap();
    assert!(mail.contains("ada@example.com"));
    assert!(mail.contains("/login/verify?token="));

    let verify = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/login/verify?token={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(verify.status(), StatusCode::SEE_OTHER);
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

    let admin = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri("/admin")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(admin.status(), StatusCode::OK);
    let admin_body = to_bytes(admin.into_body(), usize::MAX).await.unwrap();
    assert!(
        String::from_utf8(admin_body.to_vec())
            .unwrap()
            .contains("ada@example.com")
    );

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
    assert_eq!(logout.status(), StatusCode::SEE_OTHER);
    let after_logout = router
        .handle(
            Request::builder()
                .method(Method::GET)
                .uri("/admin")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(after_logout.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(after_logout.headers().get("location").unwrap(), "/login");

    fs::remove_dir_all(mail_dir).unwrap();
}
// ANCHOR_END: auth-integration-test
