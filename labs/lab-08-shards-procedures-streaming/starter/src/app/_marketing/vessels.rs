use crate::shared::VESSELS;
use topcoat::{
    Result,
    router::page,
    runtime::shard,
    view::{View, view},
};

#[shard]
pub(crate) async fn vessel_results(query: String) -> Result<impl View> {
    // TODO(lab-08): validate `query`, filter VESSELS on the server, and render
    // the matching rows plus empty-query and no-results states.
    let _ = (&query, VESSELS);
    Ok(view! { <p data-component="vessel-results">"Search results go here."</p> })
}

#[page]
async fn vessels() -> Result<impl View> {
    // TODO(lab-08): declare a query signal, bind an input with @input/:value,
    // and pass `$(query.get())` into `vessel_results`.
    let query = String::new();
    Ok(view! {
        <h1>"Vessels"</h1>
        vessel_results(query: $(query))
    })
}
