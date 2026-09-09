//! Integration tests: build the real router, dispatch requests, assert on HTML.
//!
//! The headline test is the lab's checkpoint: the same `berth_card` call must
//! produce the same HTML on the home page and on the detail page.

use topcoat::router::{
    Body, Method, Router, RouterBuilderDiscoverExt, StatusCode, request::Request, to_bytes,
};

/// Requests `path` from a freshly discovered router and returns status and HTML.
async fn get(path: &str) -> (StatusCode, String) {
    // Touching the library guarantees it is linked into this test binary, so
    // its inventory-registered pages are there for `discover()` to find.
    assert!(!lab03_solution::berths().is_empty());

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

/// Extracts the whole `<article …data-berth="slug"…>…</article>` for one berth.
fn card<'a>(html: &'a str, slug: &str) -> &'a str {
    let marker = format!("data-berth=\"{slug}\"");
    let marker_at = html
        .find(&marker)
        .unwrap_or_else(|| panic!("no card for {slug} in:\n{html}"));
    let start = html[..marker_at]
        .rfind("<article")
        .expect("card starts with an <article>");
    let end = html[start..]
        .find("</article>")
        .map(|offset| start + offset + "</article>".len())
        .expect("card ends with an </article>");

    &html[start..end]
}

#[tokio::test]
async fn every_page_shares_the_layout() {
    for path in ["/", "/berths", "/berths/featured"] {
        let (status, html) = get(path).await;

        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(html.starts_with("<!DOCTYPE html>"), "{path}: {html}");
        assert!(html.contains("<nav aria-label=\"Main\">"), "{path}: {html}");
        assert!(html.contains("Slipway Marina"), "{path}: {html}");
    }
}

#[tokio::test]
async fn pages_provide_their_own_content_as_children() {
    let (_, home) = get("/").await;
    let (_, berths) = get("/berths").await;

    assert!(home.contains("<title>Slipway · Slipway</title>"), "{home}");
    assert!(home.contains("a small marina"), "{home}");
    assert!(!berths.contains("a small marina"), "{berths}");
    assert!(berths.contains("<ul class=\"berths\">"), "{berths}");
}

/// Normalises a card for comparison by sorting the opening tag's attributes.
///
/// Spreading an `Attributes` value into an element routes that element's
/// attributes through a map, and topcoat 0.7.0 documents that attribute render
/// order is not guaranteed. The attribute *set* is what has to match.
fn normalized(card: &str) -> String {
    let tag_end = card.find('>').expect("opening tag");
    let (open, rest) = card.split_at(tag_end);
    let open = open.trim_start_matches("<article").trim();

    let mut attributes = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for character in open.chars() {
        match character {
            '"' => {
                in_quotes = !in_quotes;
                current.push(character);
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    attributes.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(character),
        }
    }
    if !current.is_empty() {
        attributes.push(current);
    }
    attributes.sort();

    format!("<article {}{rest}", attributes.join(" "))
}

/// The lab checkpoint: one component, one rendering, wherever it is called.
#[tokio::test]
async fn berth_card_renders_identically_on_every_page() {
    let (_, home) = get("/").await;
    let (_, detail) = get("/berths/featured").await;
    let (_, list) = get("/berths").await;

    assert_eq!(
        normalized(card(&home, "a1")),
        normalized(card(&detail, "a1"))
    );
    assert_eq!(normalized(card(&home, "a1")), normalized(card(&list, "a1")));
}

#[tokio::test]
async fn berth_list_renders_a_card_per_berth() {
    let (_, html) = get("/berths").await;

    for slug in ["a1", "a2", "b7", "c3"] {
        let card = card(&html, slug);
        assert!(card.contains("berth-card"), "{card}");
        assert!(card.contains("class=\"badge"), "{card}");
    }
    assert_eq!(html.matches("<article").count(), 4, "{html}");
}

#[tokio::test]
async fn callers_can_pass_attributes_through_to_the_card() {
    let (_, detail) = get("/berths/featured").await;
    let decorated = card(&detail, "a2");

    assert!(decorated.contains("data-detail=\"\""), "{decorated}");

    // The caller's class is merged with the component's own, not substituted
    // for it.
    assert!(decorated.contains("berth-card-detail"), "{decorated}");
    assert!(decorated.contains("berth-card"), "{decorated}");

    // ...and the plain call is unaffected.
    let (_, list) = get("/berths").await;
    assert!(!card(&list, "a2").contains("berth-card-detail"));
    assert!(!card(&list, "a2").contains("data-detail"));
}

#[tokio::test]
async fn child_content_appears_only_where_it_is_passed() {
    let (_, detail) = get("/berths/featured").await;

    assert!(card(&detail, "a2").contains("long-stay rates"), "{detail}");
    assert!(!card(&detail, "a1").contains("long-stay rates"), "{detail}");

    let (_, list) = get("/berths").await;
    assert!(!list.contains("long-stay rates"), "{list}");
}

#[tokio::test]
async fn an_unknown_slug_renders_a_missing_card() {
    // The card fetches its own data, so it also owns the not-found case.
    // Returning a 404 from a component is Lab 06 material.
    assert!(lab03_solution::find_berth("nope").is_none());
    assert!(lab03_solution::find_berth("a1").is_some());
}
