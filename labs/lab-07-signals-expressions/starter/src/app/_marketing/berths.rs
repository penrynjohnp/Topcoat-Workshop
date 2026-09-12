mod id;

use crate::shared::{berth_card, berths};
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

#[page]
#[rustfmt::skip]
async fn list(cx: &Cx) -> Result<impl View> {
    let data = berths();
    let _ = cx;
    // TODO(lab-07): import `signal` from `topcoat::runtime`, then create
    // `query`, `show_occupied`, and `show_vacant` here with
    // `signal(cx, || initial_value)` before entering `view!`.

    Ok(view! {
        <h1>"Berths"</h1>

        // TODO(lab-07): add the filter form — a search input bound with `:value` and
        // `@input`, and two chips that `toggle` their signal and bind `:class`.
        <form class="berth-filter" data-component="berth-filter">
            <label for="berth-query">"Filter by name"</label>
            <input id="berth-query" type="search" placeholder="A1">
        </form>

        <ul class="berths">
            for berth in data {
                // TODO(lab-07): capture the berth name and status, then hide the row
                // with `:hidden` when it matches neither the query nor the chips.
                // Remember: `&&` and `||` are not in the expression vocabulary.
                <li data-berth=(berth.slug)>berth_card(slug: berth.slug)</li>
            }
        </ul>
    })
}
