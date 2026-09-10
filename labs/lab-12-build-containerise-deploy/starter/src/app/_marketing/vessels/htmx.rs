mod results;

use crate::{
    assets,
    components::{button::button, input::input, label::label},
    shared::vessel_matches,
};
use topcoat::{
    Result,
    context::Cx,
    icon::icon,
    router::{page, query_params},
    view::{View, attributes, view},
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
        <div class="flex items-center gap-3">
            <span class="rounded-xl bg-cyan-100 p-2 text-cyan-800">
                icon(data: assets::SEARCH)
            </span>
            <div>
                <h1 class="text-3xl font-bold tracking-tight text-cyan-950">
                    "Vessels (htmx)"
                </h1>
                <p class="text-slate-600">
                    "The progressively enhanced version of the same search."
                </p>
            </div>
        </div>
        <p class="mt-5 text-sm text-slate-600">
            "The same search as "
            <a href="/vessels">"/vessels"</a>
            ", driven by htmx attributes instead of a shard."
        </p>

        // A real GET form, so the page still works with JavaScript off.
        // htmx intercepts the input before the form is ever submitted.
        <form
            method="get"
            action="/vessels/htmx"
            class="mt-7 grid max-w-2xl gap-2 sm:grid-cols-[1fr_auto] sm:items-end"
        >
            <div class="grid gap-2">
                label(attrs: attributes! { for="vessel-query-htmx" }, "Search vessels")
                input(
                    attrs: attributes! {
                        id="vessel-query-htmx"
                        type="search"
                        name="q"
                        value=(&query)
                        hx-get="/vessels/htmx/results"
                        hx-trigger="input changed delay:250ms, search"
                        hx-target="#vessel-results-htmx"
                        hx-indicator="#vessel-spinner"
                    }
                )
            </div>
            button(
                attrs: attributes! { type="submit" },
                icon(data: assets::SEARCH)
                "Search"
            )
            <span id="vessel-spinner" class="htmx-indicator text-sm text-cyan-800">
                "Searching…"
            </span>
        </form>

        // Where a retargeted validation failure lands.
        <p id="vessel-alert" role="alert"></p>

        <div id="vessel-results-htmx">vessel_matches(query: &query)</div>
    })
}
// ANCHOR_END: htmx-search-page
