use crate::shared::{self, load_berth};
use topcoat::{
    Result,
    context::Cx,
    router::{page, path_param},
    view::{View, component, view},
};

path_param!(id);

// ANCHOR: memoized-components
#[component]
async fn berth_title(cx: &Cx, slug: &str) -> Result<impl View> {
    let Some(berth) = load_berth(cx, slug).await else {
        return Err(topcoat::router::error::not_found().into());
    };
    Ok(view! {
        <section data-component="berth-title" data-berth=(berth.slug)>
            <h2>
                "Berth "
                (berth.name)
            </h2>
        </section>
    })
}

#[component]
async fn berth_summary(cx: &Cx, slug: &str) -> Result<impl View> {
    let Some(berth) = load_berth(cx, slug).await else {
        return Err(topcoat::router::error::not_found().into());
    };
    Ok(view! {
        <section data-component="berth-summary">
            <p>
                (berth.length_m)
                "m · "
                (berth.vessel().unwrap_or("Available"))
            </p>
        </section>
    })
}

#[component]
async fn berth_note(cx: &Cx, slug: &str) -> Result<impl View> {
    let Some(berth) = load_berth(cx, slug).await else {
        return Err(topcoat::router::error::not_found().into());
    };
    Ok(view! {
        <section data-component="berth-note">
            <p>
                "Vessel: "
                (berth.vessel().unwrap_or("No vessel assigned"))
            </p>
        </section>
    })
}
// ANCHOR_END: memoized-components

#[page]
async fn detail(cx: &Cx) -> Result<impl View> {
    let id = path_param::<Id>(cx);
    if shared::find_berth(id).is_none() {
        return Err(topcoat::router::error::not_found().into());
    }
    Ok(view! {
        <h1>"Berth detail"</h1>
        berth_title(slug: id)
        berth_summary(slug: id)
        berth_note(slug: id)
    })
}
