use crate::shared::VESSELS;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    runtime::{Event, Signal, shard, signal},
    view::{View, view},
};

// ANCHOR: vessel-results-shard
#[shard]
pub(crate) async fn vessel_results(query: Signal<String>) -> Result<impl View> {
    let query = query.get();
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
                    // ANCHOR: vessel-result-ids
                    for vessel in matches {
                        let vessel_id = format!(
                            "vessel-{}",
                            vessel.name.to_ascii_lowercase().replace(' ', "-"),
                        );
                        <li id=(vessel_id) data-vessel=(vessel.name)>
                            <strong>(vessel.name)</strong>
                            " · "
                            (vessel.berth)
                        </li>
                    } // ANCHOR_END: vessel-result-ids
                </ul>
            }
        </section>
    })
}
// ANCHOR_END: vessel-results-shard

// ANCHOR: vessel-search-page
#[page]
async fn vessels(cx: &Cx) -> Result<impl View> {
    let query = signal(cx, String::new);

    Ok(view! {
        <h1>"Vessels"</h1>
        <label for="vessel-query">"Search vessels"</label>
        <input
            id="vessel-query"
            type="search"
            :value=$(query.get())
            @input=$(|event: Event| query.set(event.target.value))
        >
        vessel_results(query: $(query))
    })
}
// ANCHOR_END: vessel-search-page
