use crate::shared::VESSELS;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    runtime::{Event, Signal, shard, signal},
    view::{View, view},
};

#[shard]
pub(crate) async fn vessel_results(query: Signal<String>) -> Result<impl View> {
    // TODO(lab-08): validate the signal value, filter VESSELS on the server, and render
    // the matching rows plus the empty-query and no-results states. Give each result
    // a stable HTML `id` so morphing can follow an item if the list reorders.
    let _ = (&query, VESSELS);
    Ok(view! { <p data-component="vessel-results">"Search results go here."</p> })
}

#[page]
async fn vessels(cx: &Cx) -> Result<impl View> {
    // TODO(lab-08): create a query signal with `signal(cx, String::new)`, bind an input
    // with `@input` and `:value`, then pass the signal itself into `vessel_results` as `$(query)`.
    let query = signal(cx, String::new);
    Ok(view! {
        <h1>"Vessels"</h1>
        <input
            type="search"
            :value=$(query.get())
            @input=$(|event: Event| query.set(event.target.value))
        >
        vessel_results(query: $(query))
    })
}
