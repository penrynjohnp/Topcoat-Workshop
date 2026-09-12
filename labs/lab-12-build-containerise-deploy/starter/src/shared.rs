use std::sync::atomic::Ordering;

use crate::{
    auth::AppState,
    components::card::{card, card_content, card_header, card_title},
    models::{Berth, Vessel, WorkOrder},
};
use topcoat::{
    Result,
    context::{Cx, app_context, memoize},
    view::{Attributes, Child, View, ViewExt, class, component, view},
};

pub const FEATURED: &str = "a1";
pub const NAV: [(&str, &str); 4] = [
    ("/", "Home"),
    ("/berths", "Berths"),
    ("/vessels", "Vessels"),
    ("/vessels/htmx", "Vessels (htmx)"),
];
pub const MAX_QUERY_LEN: usize = 80;

fn db(cx: &Cx) -> toasty::Db {
    app_context::<AppState>(cx).db.clone()
}

// ANCHOR: toasty-queries
pub async fn managed_berths(cx: &Cx) -> Result<Vec<Berth>> {
    let mut db = db(cx);
    Ok(Berth::filter_by_managed(true)
        .include(Berth::fields().vessels())
        .exec(&mut db)
        .await?)
}

pub async fn search_vessels(cx: &Cx, query: &str) -> Result<Vec<Vessel>> {
    let query = query.trim();
    if query.len() > MAX_QUERY_LEN {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "query too long").into());
    }
    let mut db = db(cx);
    let vessels = if query.is_empty() {
        Vessel::all()
            .include(Vessel::fields().berth())
            .exec(&mut db)
            .await?
    } else {
        Vessel::filter(Vessel::fields().name().like(format!("%{query}%")))
            .include(Vessel::fields().berth())
            .exec(&mut db)
            .await?
    };
    Ok(vessels)
}

pub async fn work_orders(cx: &Cx) -> Result<Vec<WorkOrder>> {
    let mut db = db(cx);
    Ok(WorkOrder::all().exec(&mut db).await?)
}

pub async fn work_order_exists(cx: &Cx, id: u64) -> Result<bool> {
    let mut db = db(cx);
    Ok(WorkOrder::filter_by_id(id)
        .first()
        .exec(&mut db)
        .await?
        .is_some())
}
// ANCHOR_END: toasty-queries

#[memoize(as_ref)]
pub async fn load_berth(cx: &Cx, slug: &str) -> Option<Berth> {
    let state: &AppState = app_context(cx);
    state.load_count.fetch_add(1, Ordering::Relaxed);
    let mut db = state.db.clone();
    Berth::filter_by_slug(slug)
        .include(Berth::fields().vessels())
        .first()
        .exec(&mut db)
        .await
        .ok()
        .flatten()
}

#[component]
pub(crate) async fn vessel_matches(cx: &Cx, query: &str) -> Result<impl View> {
    let query = query.trim();
    let matches = search_vessels(cx, query).await?;
    Ok(view! {
        <section data-component="vessel-results" data-query=(query) class="mt-6">
            if query.is_empty() {
                <p>"Type a vessel name, or browse every vessel below."</p>
            }
            if matches.is_empty() {
                <p class="empty">
                    "No vessels match "
                    <strong>(query)</strong>
                    "."
                </p>
            } else {
                <ul class="grid gap-3 sm:grid-cols-2">
                    for vessel in matches {
                        let vessel_id = format!(
                            "vessel-{}",
                            vessel.name.to_ascii_lowercase().replace(' ', "-"),
                        );
                        <li
                            id=(vessel_id)
                            data-vessel=(&vessel.name)
                            class="rounded-xl border border-cyan-900/10 bg-white/80 p-4 shadow-xs"
                        >
                            <strong>(&vessel.name)</strong>
                            " · "
                            (&vessel.berth.get().name)
                        </li>
                    }
                </ul>
            }
        </section>
    })
}

#[component]
pub(crate) async fn nav(current_path: &str) -> Result<impl View> {
    Ok(view! {
        <nav aria-label="Main">
            <ul class="nav flex flex-wrap items-center gap-1">
                for (href, label) in NAV {
                    <li>
                        <a
                            href=(href)
                            class=(class!(
                                "nav-link rounded-lg px-3 py-2 text-sm font-medium text-slate-600 hover:bg-cyan-50 hover:text-cyan-900",
                                "nav-link-active bg-cyan-100 text-cyan-950" if href
                                    == current_path,
                            ))
                            aria-current=((href == current_path).then_some("page"))
                        >
                            (label)
                        </a>
                    </li>
                }
            </ul>
        </nav>
    })
}

#[component]
pub(crate) async fn site_footer() -> Result<impl View> {
    Ok(view! {
        <footer
            class="footer border-t border-cyan-900/10 bg-cyan-950 px-5 py-8 text-center text-sm text-cyan-50"
        >
            <p>"Slipway Marina · Built with Topcoat"</p>
        </footer>
    })
}

#[component]
pub(crate) async fn status_badge(status: &str, vessel: Option<String>) -> Result<impl View> {
    Ok(view! {
        if status == "vacant" {
            <span
                class=(class!(
                    "badge inline-flex rounded-full bg-emerald-100 px-2.5 py-1 text-xs font-semibold text-emerald-800",
                    "badge-vacant",
                ))
            >
                "Vacant"
            </span>
        } else if status == "occupied" {
            <span
                class=(class!(
                    "badge inline-flex rounded-full bg-sky-100 px-2.5 py-1 text-xs font-semibold text-sky-800",
                    "badge-occupied",
                ))
            >
                "Occupied by "
                (vessel.as_deref().unwrap_or("an unlisted vessel"))
            </span>
        } else {
            <span
                class=(class!(
                    "badge inline-flex rounded-full bg-amber-100 px-2.5 py-1 text-xs font-semibold text-amber-900",
                    "badge-maintenance",
                ))
            >
                "Maintenance"
            </span>
        }
    })
}

#[component]
#[rustfmt::skip]
pub(crate) async fn berth_card(
    cx: &Cx,
    slug: &str,
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let mut attrs = attrs;
    let caller_class = attrs.remove("class");
    let mut db = db(cx);
    let Some(berth) = Berth::filter_by_slug(slug)
        .include(Berth::fields().vessels())
        .first()
        .exec(&mut db)
        .await?
    else {
        return Ok(view! {
            card(
                attrs: topcoat::view::attributes! { class="berth-card berth-card-missing" },
                card_content(
                    <p>
                        "No berth "
                        (slug)
                        "."
                    </p>
                )
            )
        }.boxed());
    };
    let vessel = berth.vessels.get().first().map(|vessel| vessel.name.clone());
    Ok(view! {
        card(
            attrs: topcoat::view::attributes! { class=(class!("berth-card h-full", caller_class)) },
            <article
                data-ui="marina-card"
                data-berth=(&berth.slug)
                data-length=(berth.length_m.to_string())
                (attrs)
            >
                card_header(
                    card_title(
                        attrs: topcoat::view::attributes! { class="text-xl text-cyan-950" },
                        <a href=(format!("/berths/{}", berth.slug))>
                            "Berth "
                            (&berth.name)
                        </a>
                    )
                )
                card_content(
                    attrs: topcoat::view::attributes! { class="flex items-center justify-between gap-4" },
                    <p class="berth-length text-sm font-medium text-slate-500">
                        (berth.length_m)
                        " metres"
                    </p>
                    status_badge(status: &berth.status, vessel: vessel)
                    (child)
                )
            </article>
        )
    }.boxed())
}

#[component]
pub(crate) async fn berth_card_from_model(berth: &Berth) -> Result<impl View> {
    let vessel = berth
        .vessels
        .get()
        .first()
        .map(|vessel| vessel.name.clone());
    Ok(view! {
        card(
            attrs: topcoat::view::attributes! { class="berth-card h-full" },
            <article
                data-ui="marina-card"
                data-berth=(&berth.slug)
                data-length=(berth.length_m.to_string())
            >
                card_header(
                    card_title(
                        attrs: topcoat::view::attributes! { class="text-xl text-cyan-950" },
                        <a href=(format!("/berths/{}", berth.slug))>
                            "Berth "
                            (&berth.name)
                        </a>
                    )
                )
                card_content(
                    attrs: topcoat::view::attributes! { class="flex items-center justify-between gap-4" },
                    <p class="berth-length text-sm font-medium text-slate-500">
                        (berth.length_m)
                        " metres"
                    </p>
                    status_badge(status: &berth.status, vessel: vessel)
                )
            </article>
        )
    })
}

#[component]
#[rustfmt::skip]
pub(crate) async fn berth_list(cx: &Cx) -> Result<impl View> {
    let data = managed_berths(cx).await?;
    Ok(view! {
        if data.is_empty() {
            <p class="empty">"No berths yet."</p>
        } else {
            <ul class="berths grid gap-5 md:grid-cols-2">
                for berth in data {
                    <li class="h-full">berth_card_from_model(berth: &berth)</li>
                }
            </ul>
        }
    })
}
