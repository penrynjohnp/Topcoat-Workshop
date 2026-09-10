mod results;

use crate::shared::vessel_matches;
use topcoat::{
    Result,
    context::Cx,
    router::{page, query_params},
    view::{View, view},
};

// ANCHOR: htmx-query-params
/// Typed, memoized access to `?q=`. Lab 08 hand-split the query string; this is
/// the real API, and it is shared with the fragment route below.
#[query_params(error = redirect("?"))]
pub(crate) struct VesselQuery {
    pub q: Option<String>,
}
// ANCHOR_END: htmx-query-params

// ANCHOR: htmx-search-page
#[page]
async fn vessels_htmx(cx: &Cx) -> Result<impl View> {
    // The page renders the first results on the server. That is what makes the
    // URL htmx pushes honest: reloading a pushed `?q=…` shows the same list.
    let query = query_params::<VesselQuery>(cx)?
        .q
        .clone()
        .unwrap_or_default();

    Ok(view! {
        <h1>"Vessels (htmx)"</h1>
        <p>
            "The same search as "
            <a href="/vessels">"/vessels"</a>
            ", driven by htmx attributes instead of a shard."
        </p>

        // A real GET form, so the page still works with JavaScript off.
        // htmx intercepts the input before the form is ever submitted.
        <form method="get" action="/vessels/htmx">
            <label for="vessel-query-htmx">"Search vessels"</label>
            <input
                id="vessel-query-htmx"
                type="search"
                name="q"
                value=(&query)
                hx-get="/vessels/htmx/results"
                hx-trigger="input changed delay:250ms, search"
                hx-target="#vessel-results-htmx"
                hx-indicator="#vessel-spinner"
            >
            <button type="submit">"Search"</button>
            <span id="vessel-spinner" class="htmx-indicator">"Searching…"</span>
        </form>

        // Where a retargeted validation failure lands.
        <p id="vessel-alert" role="alert"></p>

        <div id="vessel-results-htmx">vessel_matches(query: &query)</div>
    })
}
// ANCHOR_END: htmx-search-page
