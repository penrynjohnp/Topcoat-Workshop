use crate::{
    app::_marketing::vessels::htmx::VesselQuery,
    shared::{MAX_QUERY_LEN, search_vessels, vessel_matches},
};
use topcoat::{
    Result,
    context::Cx,
    htmx::{HxPushUrl, HxResponseTrigger, HxReswap, HxRetarget, SwapOption, hx_request},
    router::{
        error::redirect,
        query_params,
        response::{IntoResponse, Response},
        route,
    },
    view::{ViewExt, view},
};

// ANCHOR: htmx-fragment-route
#[route(GET)]
async fn results(cx: &Cx) -> Result<Response> {
    // A fragment endpoint is as public as a shard endpoint. The transport
    // changed; the threat model did not. Validate the input, and send a human
    // who opened this URL somewhere that makes sense.
    if !hx_request(cx) {
        return Err(redirect("/vessels/htmx").into());
    }

    let query = query_params::<VesselQuery>(cx)?
        .q
        .clone()
        .unwrap_or_default();

    let Ok(matches) = search_vessels(&query) else {
        // Failure belongs in the alert region, not the results region. The
        // server overrides the client's `hx-target` and `hx-swap` for this one
        // response — markup the client never has to know about.
        let alert = view! {
            <span data-vessel-error="too-long">
                "Search text is longer than "
                (MAX_QUERY_LEN)
                " characters."
            </span>
        };
        return (
            HxRetarget::from("#vessel-alert"),
            HxReswap(SwapOption::InnerHtml),
            alert.single().await?,
        )
            .into_response(cx);
    };

    // Push the search into the address bar so it is shareable and the back
    // button works, and announce the result count for anything listening.
    let pushed = format!("/vessels/htmx?q={}", urlencode(&query));
    let trigger = HxResponseTrigger::after_swap([topcoat::htmx::HxEvent::with_data(
        "vessels:searched",
        matches.len(),
    )?]);

    (
        HxPushUrl(pushed),
        trigger,
        view! { vessel_matches(query: &query) }.single().await?,
    )
        .into_response(cx)
}
// ANCHOR_END: htmx-fragment-route

fn urlencode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            b' ' => "+".to_owned(),
            other => format!("%{other:02X}"),
        })
        .collect()
}
