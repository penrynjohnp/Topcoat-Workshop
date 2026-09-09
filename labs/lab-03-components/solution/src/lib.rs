//! Slipway, composed from components — the Lab 03 solution.
//!
//! Same markup as Lab 02, but the document shell is a `layout` component with
//! child content, and the berth card is its own component that fetches the
//! berth it renders.

use topcoat::{
    Result,
    router::page,
    view::{Attributes, Child, View, ViewExt, attributes, class, component, view},
};

/// The navigation links, in order.
const NAV: [(&str, &str); 2] = [("/", "Home"), ("/berths", "Berths")];

/// The berth shown on the home page and the detail page.
const FEATURED: &str = "a1";

/// A berth in the marina.
pub struct Berth {
    pub name: &'static str,
    pub slug: &'static str,
    pub length_m: u8,
    pub status: Status,
}

/// What is currently happening at a berth.
pub enum Status {
    Vacant,
    Occupied { vessel: &'static str },
    Maintenance,
}

impl Berth {
    /// The berth's detail URL. Lab 04 turns this into a real route.
    pub fn url(&self) -> String {
        format!("/berths/{}", self.slug)
    }
}

/// Every berth. Lab 10 replaces this with a Toasty query.
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

/// One berth by slug. In Lab 10 this becomes a query; today it is a scan.
pub fn find_berth(slug: &str) -> Option<Berth> {
    berths().into_iter().find(|berth| berth.slug == slug)
}

// ANCHOR: layout
/// The whole HTML document. Pages pass their own markup as child content, so
/// the shell lives in exactly one place.
#[component]
async fn layout(title: &str, current_path: &str, #[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>
                    (title)
                    " · Slipway"
                </title>
                topcoat::dev::script()
            </head>
            <body>
                <header>
                    <h1>(title)</h1>
                    site_nav(current_path: current_path)
                </header>
                <main>(child)</main>
                site_footer()
            </body>
        </html>
    })
}
// ANCHOR_END: layout

/// The site navigation. Lab 05 reads the current path from `Cx` instead of
/// taking it as an argument.
#[component]
async fn site_nav(current_path: &str) -> Result<impl View> {
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

/// The site footer.
#[component]
async fn site_footer() -> Result<impl View> {
    Ok(view! {
        <footer class="footer"><p>"Slipway Marina · Built with Topcoat"</p></footer>
    })
}

/// A status badge.
#[component]
async fn status_badge(status: &Status) -> Result<impl View> {
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

// ANCHOR: berth_card
/// One berth, rendered the same way everywhere it appears.
///
/// The card takes a slug, not a `Berth`: it fetches the data it renders. That
/// is locality of behaviour — every caller needs the slug it already has, and
/// nobody has to thread a `Berth` down to reach this component.
///
/// `attrs` lets a caller decorate the card without this component knowing why,
/// and `child` gives it an optional footer slot.
#[component]
async fn berth_card(
    slug: &str,
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    // The caller's classes are merged with the card's own rather than
    // overwriting them.
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
        }
        .boxed());
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
    }
    .boxed())
}
// ANCHOR_END: berth_card

// ANCHOR: berth_list
/// The berth list. It fetches its own berths and delegates each card, so the
/// pages that use it pass nothing at all.
#[component]
async fn berth_list() -> Result<impl View> {
    let berths = berths();

    Ok(view! {
        if berths.is_empty() {
            <p class="empty">"No berths yet."</p>
        } else {
            <ul class="berths">
                for berth in berths {
                    <li>berth_card(slug: berth.slug)</li>
                }
            </ul>
        }
    })
}
// ANCHOR_END: berth_list

// ANCHOR: pages
#[page("/")]
async fn home() -> Result<impl View> {
    Ok(view! {
        layout(
            title: "Slipway",
            current_path: "/",
            <p>"Berths, vessels and work orders for a small marina."</p>
            <h2>"Featured berth"</h2>
            berth_card(slug: FEATURED)
        )
    })
}

#[page("/berths")]
async fn berths_page() -> Result<impl View> {
    Ok(view! { layout(title: "Berths", current_path: "/berths", berth_list()) })
}

#[page("/berths/featured")]
async fn featured_berth_page() -> Result<impl View> {
    Ok(view! {
        layout(
            title: "Featured berth",
            current_path: "/berths",
            // Exactly the call the home page makes, so the card renders
            // byte-for-byte the same on both pages.
            berth_card(slug: FEATURED)
            <h2>"Next door"</h2>
            // The same component, decorated by the caller: extra attributes and
            // a note in the card's child slot.
            berth_card(
                slug: "a2",
                attrs: attributes! { class="berth-card-detail" data-detail="" },
                <p class="berth-note">"Ask at the office about long-stay rates."</p>
            )
        )
    })
}
// ANCHOR_END: pages
