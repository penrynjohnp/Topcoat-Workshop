use crate::shared::VESSELS;
use topcoat::{
    Result,
    router::page,
    runtime::{Event, shard},
    view::{View, view},
};

// ANCHOR: vessel-results-shard
#[shard]
pub(crate) async fn vessel_results(query: String) -> Result<impl View> {
    let query = query.trim().to_owned();
    let too_long = query.len() > 80;
    let matches = VESSELS
        .iter()
        .filter(|vessel| !too_long && (query.is_empty() || vessel.name.contains(&query)))
        .collect::<Vec<_>>();

    Ok(view! {
        <section data-component="vessel-results" data-query=(&query)>
            if too_long {
                <p class="error">"Search text is too long."</p>
            } else if query.is_empty() {
                <p>"Type a vessel name, or browse every vessel below."</p>
            }
            if !too_long && matches.is_empty() {
                <p class="empty">
                    "No vessels match "
                    <strong>(&query)</strong>
                    "."
                </p>
            } else {
                <ul>
                    for vessel in matches {
                        <li data-vessel=(vessel.name)>
                            <strong>(vessel.name)</strong>
                            " · "
                            (vessel.berth)
                        </li>
                    }
                </ul>
            }
        </section>
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
