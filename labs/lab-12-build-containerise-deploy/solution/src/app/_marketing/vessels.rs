mod htmx;

use crate::{
    assets,
    components::{input::input, label::label},
    shared::{MAX_QUERY_LEN, vessel_matches},
};
use topcoat::{
    Result,
    context::Cx,
    icon::icon,
    router::page,
    runtime::{Event, shard},
    view::{View, attributes, view},
};

// ANCHOR: vessel-results-shard
#[shard]
pub(crate) async fn vessel_results(cx: &Cx, query: String) -> Result<impl View> {
    let _ = cx;
    let query = query.trim().to_owned();
    let too_long = query.len() > MAX_QUERY_LEN;

    Ok(view! {
        if too_long {
            <p class="error">"Search text is too long."</p>
        } else {
            vessel_matches(query: &query)
        }
    })
}
// ANCHOR_END: vessel-results-shard

// ANCHOR: vessel-search-page
#[page]
async fn vessels() -> Result<impl View> {
    Ok(view! {
        signal query = String::new();

        <div class="flex items-center gap-3">
            <span class="rounded-xl bg-cyan-100 p-2 text-cyan-800">
                icon(data: assets::SEARCH)
            </span>
            <div>
                <h1 class="text-3xl font-bold tracking-tight text-cyan-950">
                    "Vessels"
                </h1>
                <p class="text-slate-600">
                    "Search the marina register with a native shard."
                </p>
            </div>
        </div>
        <div class="mt-7 grid max-w-xl gap-2">
            label(attrs: attributes! { for="vessel-query" }, "Search vessels")
            input(
                attrs: attributes! {
                    id="vessel-query"
                    type="search"
                    placeholder="Lady Jane"
                    :value=$(query.get())
                    @input=$(|event: Event| query.set(event.target.value))
                }
            )
        </div>
        vessel_results(query: $(query.get()))
    })
}
// ANCHOR_END: vessel-search-page
