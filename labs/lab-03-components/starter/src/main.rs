//! Lab 03 starter — this is the Lab 02 solution, unchanged.
//!
//! Nothing here has been broken on purpose. Work through the TODOs in order to
//! refactor it; each one matches a step in the lab README.
//! `grep -rn "TODO(lab-03)" src` lists them all.

use topcoat::{
    Result,
    router::{Router, RouterBuilderDiscoverExt, page},
    view::{View, attributes, class, component, view},
};

#[tokio::main]
async fn main() {
    topcoat::start(Router::builder().discover().build())
        .await
        .unwrap();
}

/// The navigation links, in order. A plain array so `nav` can loop over it.
const NAV: [(&str, &str); 2] = [("/", "Home"), ("/berths", "Berths")];

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

/// The site footer.
#[component]
async fn site_footer() -> Result<impl View> {
    Ok(view! {
        <footer class="footer"><p>"Slipway Marina · Built with Topcoat"</p></footer>
    })
}

// TODO(lab-03) step 2: add a `layout` component above this one that owns the
// whole document — the doctype line down to the footer — and takes
// `title: &str`, `current_path: &str` and `#[default] child: Child<'_>`.
// Both pages below then call it and shrink to their own markup.

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

// TODO(lab-03) step 3: lift the <article> below into a `berth_card` component
// taking `berth: &Berth`, so the card can be used outside this list.
// TODO(lab-03) step 4: then change `berth_card` to take `slug: &str` and call
// `find_berth(slug)` itself, and drop this function's `berths` argument in
// favour of calling `berths()` here. You will need to write `find_berth`.
// TODO(lab-03) step 6: give `berth_card` a `#[default] attrs: Attributes`
// parameter, spread it into the <article>, and merge the caller's `class` with
// the card's own using `attrs.remove("class")` as a `class!` entry.
// TODO(lab-03) step 7: give `berth_card` a `#[default] child: Child<'_>`
// parameter and render it at the end of the <article>.

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

// TODO(lab-03) step 5: add a `#[page("/berths/featured")]` detail page that
// calls `berth_card` with the same slug the home page uses. The card's HTML
// must come out byte-identical on both pages — that is the checkpoint.
// TODO(lab-03) step 8: try the optional-prop attributes — `#[default(expr)]`
// and `#[into]` — on one of `berth_card`'s parameters.
