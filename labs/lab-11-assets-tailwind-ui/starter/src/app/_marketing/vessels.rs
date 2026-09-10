mod htmx;

use crate::shared::{MAX_QUERY_LEN, vessel_matches};
use topcoat::{
    Result,
    context::Cx,
    router::page,
    runtime::{Event, shard},
    view::{View, view},
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

        <h1>"Vessels"</h1>
        <label for="vessel-query">"Search vessels"</label>
        <input
            id="vessel-query"
            type="search"
            :value=$(query.get())
            @input=$(|event: Event| query.set(event.target.value))
        >
        vessel_results(query: $(query.get()))
    })
}
// ANCHOR_END: vessel-search-page
