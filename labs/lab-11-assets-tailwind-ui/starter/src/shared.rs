use std::sync::atomic::Ordering;

use crate::{
    auth::AppState,
    models::{Berth, Vessel, WorkOrder},
};
use topcoat::{
    Result,
    context::{Cx, app_context, memoize},
    view::{Attributes, Child, View, ViewExt, class, component, view},
};

// TODO(lab-11): Vendor the card component and use the owned source for berth surfaces.
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
        <section data-component="vessel-results" data-query=(query)>
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
                <ul>
                    for vessel in matches {
                        <li data-vessel=(&vessel.name)>
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
            <ul class="nav">
                for (href, label) in NAV {
                    <li>
                        <a
                            href=(href)
                            class=(class!("nav-link", "nav-link-active" if href == current_path))
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
        <footer class="footer"><p>"Slipway Marina · Built with Topcoat"</p></footer>
    })
}

#[component]
pub(crate) async fn status_badge(status: &str, vessel: Option<&str>) -> Result<impl View> {
    Ok(view! {
        if status == "vacant" {
            <span class=(class!("badge", "badge-vacant"))>"Vacant"</span>
        } else if status == "occupied" {
            <span class=(class!("badge", "badge-occupied"))>
                "Occupied by "
                (vessel.unwrap_or("an unlisted vessel"))
            </span>
        } else {
            <span class=(class!("badge", "badge-maintenance"))>"Maintenance"</span>
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
    let Some(berth) = load_berth(cx, slug).await else {
        return Ok(view! {
            <article class="berth-card berth-card-missing">
                <p>
                    "No berth "
                    (slug)
                    "."
                </p>
            </article>
        }.boxed());
    };
    let vessel = berth.vessels.get().first().map(|vessel| vessel.name.as_str());
    Ok(view! {
        <article
            class=(class!("berth-card", caller_class))
            data-berth=(&berth.slug)
            data-length=(berth.length_m.to_string())
            (attrs)
        >
            <h2>
                <a href=(format!("/berths/{}", berth.slug))>
                    "Berth "
                    (&berth.name)
                </a>
            </h2>
            <p class="berth-length">
                (berth.length_m)
                "m"
            </p>
            status_badge(status: &berth.status, vessel: vessel)
            (child)
        </article>
    }.boxed())
}

#[component]
#[rustfmt::skip]
pub(crate) async fn berth_list(cx: &Cx) -> Result<impl View> {
    let data = managed_berths(cx).await?;
    Ok(view! {
        if data.is_empty() {
            <p class="empty">"No berths yet."</p>
        } else {
            <ul class="berths">
                for berth in data {
                    <li>berth_card(slug: &berth.slug)</li>
                }
            </ul>
        }
    })
}
