mod id;

use crate::shared::{berth_card, managed_berths};
use topcoat::{
    Result,
    context::Cx,
    router::page,
    runtime::Event,
    view::{View, view},
};

// ANCHOR: berth-filter
#[page]
#[rustfmt::skip]
async fn list(cx: &Cx) -> Result<impl View> {
    let data = managed_berths(cx).await?;
    Ok(view! {
        signal query = String::new();
        signal show_occupied = true;
        signal show_vacant = true;

        <h1>"Berths"</h1>

        <form
            class="berth-filter"
            data-component="berth-filter"
            @submit=$(|e: Event| e.prevent_default())
        >
            <label for="berth-query">"Filter by name"</label>
            <input
                id="berth-query"
                type="search"
                placeholder="A1"
                :value=$(query.get())
                @input=$(|e: Event| query.set(e.target.value))
            >
            <button
                type="button"
                data-filter="occupied"
                :class=$(if show_occupied.get() { "chip chip-on" } else { "chip" })
                @click=$(|_e| show_occupied.toggle())
            >
                "Occupied"
            </button>
            <button
                type="button"
                data-filter="vacant"
                :class=$(if show_vacant.get() { "chip chip-on" } else { "chip" })
                @click=$(|_e| show_vacant.toggle())
            >
                "Vacant"
            </button>
        </form>

        <ul class="berths">
            for berth in data {
                // Captured values are snapshots taken during the server render.
                let name = berth.name.to_owned();
                let occupied = !berth.vessels.get().is_empty();
                <li
                    data-berth=(&berth.slug)
                    :hidden=$({
                        // `&&` and `||` are not in the expression vocabulary, so the
                        // combination is spelled with `if`/`else` and `let` bindings.
                        let matches_name = name.contains(query.get().trim());
                        let status_shown = if occupied {
                            show_occupied.get()
                        } else {
                            show_vacant.get()
                        };
                        if matches_name { !status_shown } else { true }
                    })
                >
                    berth_card(slug: &berth.slug)
                </li>
            }
        </ul>
    })
}
// ANCHOR_END: berth-filter
