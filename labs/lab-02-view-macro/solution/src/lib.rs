//! Slipway's static shell — the Lab 02 solution.
//!
//! Everything here is markup: no database, no request context, no reactivity.
//! Data is a `Vec<Berth>` literal so the `view!` macro is the only new thing.

use topcoat::{
    Result,
    router::page,
    view::{View, attributes, class, component, view},
};

/// The navigation links, in order. A plain array so `nav` can loop over it.
const NAV: [(&str, &str); 2] = [("/", "Home"), ("/berths", "Berths")];

// ANCHOR: model
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
// ANCHOR_END: model

/// The seed data. Lab 10 replaces this with a Toasty query.
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

// ANCHOR: nav
/// The site navigation. `current_path` is passed in by the page; Lab 05
/// reads it from `Cx` instead.
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
// ANCHOR_END: nav

/// The site footer.
#[component]
async fn site_footer() -> Result<impl View> {
    Ok(view! {
        <footer class="footer"><p>"Slipway Marina · Built with Topcoat"</p></footer>
    })
}

// ANCHOR: status_badge
/// A status badge. `match` chooses both the label and the class list.
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
// ANCHOR_END: status_badge

// ANCHOR: berth_list
/// The berth list. `if` handles the empty case, `for` renders the rest, and
/// `attributes!` builds each card's attributes as ordinary Rust data.
#[component]
async fn berth_list(berths: Vec<Berth>) -> Result<impl View> {
    Ok(view! {
        if berths.is_empty() {
            <p class="empty">"No berths yet."</p>
        } else {
            <ul class="berths">
                for berth in berths {
                    <li>
                        <article
                            (attributes! {
                                class="berth-card"
                                data-berth=(berth.slug)
                                data-length=(berth.length_m.to_string())
                            })
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
                        </article>
                    </li>
                }
            </ul>
        }
    })
}
// ANCHOR_END: berth_list

// ANCHOR: page
#[page("/")]
async fn home() -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>"Slipway"</title>
                topcoat::dev::script()
            </head>
            <body>
                <header>
                    <h1>"Slipway"</h1>
                    site_nav(current_path: "/")
                </header>
                <main>
                    <p>"Berths, vessels and work orders for a small marina."</p>
                </main>
                site_footer()
            </body>
        </html>
    })
}
// ANCHOR_END: page

#[page("/berths")]
async fn berths_page() -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>"Berths · Slipway"</title>
                topcoat::dev::script()
            </head>
            <body>
                <header>
                    <h1>"Berths"</h1>
                    site_nav(current_path: "/berths")
                </header>
                <main>berth_list(berths: berths())</main>
                site_footer()
            </body>
        </html>
    })
}
