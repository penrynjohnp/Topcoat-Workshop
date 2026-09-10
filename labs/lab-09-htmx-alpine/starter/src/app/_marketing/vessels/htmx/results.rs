use crate::{
    app::_marketing::vessels::htmx::VesselQuery,
    shared::{search_vessels, vessel_matches},
};
use topcoat::{
    Result,
    context::Cx,
    router::{
        query_params,
        response::{IntoResponse, Response},
        route,
    },
    view::{ViewExt, view},
};

// ANCHOR: htmx-fragment-route
#[route(GET)]
async fn results(cx: &Cx) -> Result<Response> {
    // TODO(lab-09): a fragment endpoint is as public as a shard endpoint. Guard
    // it with `topcoat::htmx::hx_request(cx)` and send anyone who opened this
    // URL in the address bar to `/vessels/htmx` with
    // `topcoat::router::error::redirect`.

    let query = query_params::<VesselQuery>(cx)?
        .q
        .clone()
        .unwrap_or_default();

    // TODO(lab-09): when `search_vessels` reports the query is too long, answer
    // with `HxRetarget::from("#vessel-alert")` and
    // `HxReswap(SwapOption::InnerHtml)` so the message lands in the alert region
    // instead of overwriting the results.
    let _matches = search_vessels(&query).ok().unwrap_or_default();

    // TODO(lab-09): on success, add `HxPushUrl` with `/vessels/htmx?q=…` so the
    // search is shareable, and `HxResponseTrigger::after_swap` carrying the
    // match count as `HxEvent::with_data("vessels:searched", count)`.
    view! { vessel_matches(query: &query) }
        .single()
        .await?
        .into_response(cx)
}
// ANCHOR_END: htmx-fragment-route
