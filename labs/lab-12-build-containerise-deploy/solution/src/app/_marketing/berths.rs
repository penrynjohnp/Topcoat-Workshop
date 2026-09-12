mod id;

use crate::shared::{berth_card_from_model, managed_berths};
use topcoat::{
    Result,
    context::Cx,
    router::page,
    runtime::{Event, signal},
    view::{View, view},
};

// ANCHOR: berth-filter
#[page]
#[rustfmt::skip]
async fn list(cx: &Cx) -> Result<impl View> {
    let data = managed_berths(cx).await?;
    let query = signal(cx, String::new);
    let show_occupied = signal(cx, || true);
    let show_vacant = signal(cx, || true);

    Ok(view! {
        <h1 class="text-3xl font-bold tracking-tight text-cyan-950">"Berths"</h1>
        <p class="mt-2 text-slate-600">
            "Filter the managed marina berths without a server request."
        </p>

        <form
            class="berth-filter my-7 flex flex-wrap items-end gap-3 rounded-2xl border border-cyan-900/10 bg-white/75 p-4 shadow-xs"
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
                class="h-9 min-w-52 rounded-lg border border-border bg-white px-3 text-sm shadow-xs outline-none focus-visible:ring-2 focus-visible:ring-ring"
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

        <ul class="berths grid gap-5 md:grid-cols-2">
            for berth in data {
                // Captured values are snapshots taken during the server render.
                let name = berth.name.to_owned();
                let occupied = !berth.vessels.get().is_empty();
                <li
                    data-berth=(&berth.slug)
                    class="h-full"
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
                    berth_card_from_model(berth: &berth)
                </li>
            }
        </ul>
    })
}
// ANCHOR_END: berth-filter
