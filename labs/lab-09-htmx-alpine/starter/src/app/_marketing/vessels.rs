mod htmx;

use crate::shared::{MAX_QUERY_LEN, VESSELS};
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
    let too_long = query.len() > MAX_QUERY_LEN;
    let matches = VESSELS
        .iter()
        .filter(|vessel| !too_long && (query.is_empty() || vessel.name.contains(&query)))
        .collect::<Vec<_>>();

    // TODO(lab-09): this markup has to be shared with the htmx route, so move
    // it into `shared::vessel_matches` and call that component here. Keep the
    // `too_long` branch on this side: each transport reports failure its own
    // way. Give each `<li>` a stable `id` while you are there, so the morph
    // after a re-render tracks rows instead of rewriting them.
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
