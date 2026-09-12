use std::sync::atomic::Ordering;

use crate::auth::AppState;
use topcoat::{
    Result,
    context::{app_context, memoize},
    view::{Attributes, Child, View, ViewExt, class, component, view},
};

// The memoize macro requires the parameter type to be spelled `Cx` but consumes the import.
#[allow(unused_imports)]
use topcoat::context::Cx;

pub const FEATURED: &str = "a1";
pub const NAV: [(&str, &str); 4] = [
    ("/", "Home"),
    ("/berths", "Berths"),
    ("/vessels", "Vessels"),
    ("/vessels/htmx", "Vessels (htmx)"),
];

#[derive(Clone, Copy)]
pub struct Vessel {
    pub name: &'static str,
    pub berth: &'static str,
}

pub const VESSELS: [Vessel; 4] = [
    Vessel {
        name: "Lady Jane",
        berth: "A1",
    },
    Vessel {
        name: "Sea Urchin",
        berth: "C3",
    },
    Vessel {
        name: "Morning Star",
        berth: "Visitors",
    },
    Vessel {
        name: "Blue Moon",
        berth: "Yard",
    },
];

// ANCHOR: shared-search
/// The longest search we will accept. Both transports enforce it: a shard
/// argument and an htmx query string are equally untrusted.
pub const MAX_QUERY_LEN: usize = 80;

pub struct QueryTooLong;

/// The domain logic. It knows nothing about shards, htmx or HTTP.
pub fn search_vessels(query: &str) -> Result<Vec<Vessel>, QueryTooLong> {
    let query = query.trim();
    if query.len() > MAX_QUERY_LEN {
        return Err(QueryTooLong);
    }
    Ok(VESSELS
        .iter()
        .copied()
        .filter(|vessel| query.is_empty() || vessel.name.contains(query))
        .collect())
}

/// The markup. The `#[shard]` and the htmx `#[route]` both render *this*, which
/// is why their fragments are byte-for-byte identical.
#[component]
pub(crate) async fn vessel_matches(query: &str) -> Result<impl View> {
    let query = query.trim();
    let matches = search_vessels(query).ok().unwrap_or_default();
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
// ANCHOR_END: shared-search

#[derive(Clone, Copy)]
pub struct WorkOrder {
    pub id: &'static str,
    pub title: &'static str,
}

pub const WORK_ORDERS: [WorkOrder; 2] = [
    WorkOrder {
        id: "wo-a1-antifoul",
        title: "Anti-foul A1",
    },
    WorkOrder {
        id: "wo-c3-mooring",
        title: "Replace C3 mooring line",
    },
];

pub fn work_order_exists(id: &str) -> bool {
    WORK_ORDERS.iter().any(|order| order.id == id)
}

pub struct Berth {
    pub name: &'static str,
    pub slug: &'static str,
    pub length_m: u8,
    pub status: Status,
}

pub enum Status {
    Vacant,
    Occupied { vessel: &'static str },
    Maintenance,
}

impl Berth {
    pub fn url(&self) -> String {
        format!("/berths/{}", self.slug)
    }

    pub fn vessel(&self) -> Option<&'static str> {
        match &self.status {
            Status::Occupied { vessel } => Some(*vessel),
            _ => None,
        }
    }
}

pub fn berths() -> Vec<Berth> {
    vec![
        Berth {
            name: "A1",
            slug: "a1",
            length_m: 8,
            status: Status::Occupied {
                vessel: "Lady Jane",
            },
        },
        Berth {
            name: "A2",
            slug: "a2",
            length_m: 8,
            status: Status::Vacant,
        },
        Berth {
            name: "B7",
            slug: "b7",
            length_m: 12,
            status: Status::Maintenance,
        },
        Berth {
            name: "C3",
            slug: "c3",
            length_m: 15,
            status: Status::Occupied {
                vessel: "Sea Urchin",
            },
        },
    ]
}

pub fn find_berth(slug: &str) -> Option<Berth> {
    berths().into_iter().find(|berth| berth.slug == slug)
}

// ANCHOR: memoized-loader
#[memoize(as_ref)]
pub async fn load_berth(cx: &Cx, slug: &str) -> Option<Berth> {
    let state: &AppState = app_context(cx);
    state.load_count.fetch_add(1, Ordering::Relaxed);
    find_berth(slug)
}
// ANCHOR_END: memoized-loader

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
pub(crate) async fn status_badge(status: &Status) -> Result<impl View> {
    Ok(view! {
        match status {
            Status::Vacant => {
                <span class=(class!("badge", "badge-vacant"))>"Vacant"</span>
            }
            Status::Occupied { vessel } => {
                <span class=(class!("badge", "badge-occupied"))>
                    "Occupied by "
                    (vessel)
                </span>
            }
            Status::Maintenance => {
                <span class=(class!("badge", "badge-maintenance"))>"Maintenance"</span>
            }
        }
    })
}

#[component]
#[rustfmt::skip]
pub(crate) async fn berth_card(
    slug: &str,
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let mut attrs = attrs;
    let caller_class = attrs.remove("class");
    let Some(berth) = find_berth(slug) else {
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
    Ok(view! {
        <article
            class=(class!("berth-card", caller_class))
            data-berth=(berth.slug)
            data-length=(berth.length_m.to_string())
            (attrs)
        >
            <h2>
                <a href=(berth.url())>
                    "Berth "
                    (berth.name)
                </a>
            </h2>
            <p class="berth-length">
                (berth.length_m)
                "m"
            </p>
            status_badge(status: &berth.status)
            (child)
        </article>
    }.boxed())
}

#[component]
#[rustfmt::skip]
pub(crate) async fn berth_list() -> Result<impl View> {
    let data = berths();
    Ok(
        view! {
            if data.is_empty() {
                <p class="empty">"No berths yet."</p>
            } else {
                <ul class="berths">
                    for berth in data {
                        <li>berth_card(slug: berth.slug)</li>
                    }
                </ul>
            }
        },
    )
}
