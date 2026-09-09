//! Integration tests: build the real router, dispatch a request, assert on HTML.

use topcoat::router::{
    Body, Method, Router, RouterBuilderDiscoverExt, StatusCode, request::Request, to_bytes,
};

/// Requests `path` from a freshly discovered router and returns the status and
/// the rendered HTML.
async fn get(path: &str) -> (StatusCode, String) {
    // Touching the library guarantees it is linked into this test binary, so
    // its inventory-registered pages are there for `discover()` to find.
    assert!(!lab02_solution::berths().is_empty());

    let router = Router::builder().discover().build();

    let request = Request::builder()
        .method(Method::GET)
        .uri(path)
        .body(Body::empty())
        .expect("valid request");

    let response = router.handle(request).await;
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");

    (status, String::from_utf8(body.to_vec()).expect("utf-8"))
}

#[tokio::test]
async fn home_page_renders() {
    let (status, html) = get("/").await;

    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("<h1>Slipway</h1>"), "{html}");
    assert!(html.contains("Slipway Marina"), "{html}");
}

#[tokio::test]
async fn berth_list_renders_every_berth() {
    let (status, html) = get("/berths").await;

    assert_eq!(status, StatusCode::OK);
    for name in ["A1", "A2", "B7", "C3"] {
        assert!(
            html.contains(&format!("Berth {name}")),
            "missing {name}: {html}"
        );
    }
    assert!(html.contains("href=\"/berths/b7\""), "{html}");
    assert!(html.contains("data-berth=\"c3\""), "{html}");
}

#[tokio::test]
async fn status_badges_use_the_matching_arm() {
    let (_, html) = get("/berths").await;

    assert!(html.contains("Occupied by Lady Jane"), "{html}");
    assert!(html.contains("badge-vacant"), "{html}");
    assert!(html.contains("badge-maintenance"), "{html}");
}

/// The lab checkpoint: each page marks its own nav link, and only its own.
#[tokio::test]
async fn nav_highlights_the_current_page() {
    let (_, home) = get("/").await;
    let home_link = nav_link(&home, "/");
    let berths_link = nav_link(&home, "/berths");

    assert!(home_link.contains("aria-current=\"page\""), "{home_link}");
    assert!(home_link.contains("nav-link-active"), "{home_link}");
    assert!(!berths_link.contains("aria-current"), "{berths_link}");
    assert!(!berths_link.contains("nav-link-active"), "{berths_link}");

    let (_, berths) = get("/berths").await;
    let home_link = nav_link(&berths, "/");
    let berths_link = nav_link(&berths, "/berths");

    assert!(
        berths_link.contains("aria-current=\"page\""),
        "{berths_link}"
    );
    assert!(berths_link.contains("nav-link-active"), "{berths_link}");
    assert!(!home_link.contains("aria-current"), "{home_link}");
    assert!(!home_link.contains("nav-link-active"), "{home_link}");
}

/// Extracts the opening `<a>` tag whose `href` is exactly `href`.
fn nav_link<'a>(html: &'a str, href: &str) -> &'a str {
    let needle = format!("<a href=\"{href}\"");
    let start = html
        .find(&needle)
        .unwrap_or_else(|| panic!("no link to {href} in:\n{html}"));
    let end = start
        + html[start..]
            .find('>')
            .unwrap_or_else(|| panic!("unterminated tag for {href}"));

    &html[start..end]
}
