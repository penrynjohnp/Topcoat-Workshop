//! Lab 02 starter — Slipway's static shell.
//!
//! This compiles as-is. Work through the TODOs in order; each one matches a
//! step in the lab README. `grep -rn "TODO(lab-02)" src` lists them all.

use topcoat::{
    Result,
    router::{Router, RouterBuilderDiscoverExt, page},
    view::{View, component, view},
};

/// The navigation links, in order.
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

#[tokio::main]
async fn main() {
    topcoat::start(Router::builder().discover().build())
        .await
        .unwrap();
}

#[component]
async fn site_nav(current_path: &str) -> Result<impl View> {
    // Both go away once the loop and the conditional attribute are in place.
    let _ = (current_path, NAV);

    Ok(view! {
        <nav aria-label="Main">
            <ul class="nav">
                // TODO(lab-02) step 3: replace this hard-coded link with a
                // `for (href, label) in NAV { ... }` loop over the NAV array.
                <li><a href="/" class="nav-link">"Home"</a></li>
                // TODO(lab-02) step 5: mark the current page's link with
                // `aria-current=((href == current_path).then_some("page"))`.
                // TODO(lab-02) step 6: build the link's classes with
                // `class!("nav-link", "nav-link-active" if href == current_path)`.
                <li><a href="/berths" class="nav-link">"Berths"</a></li>
            </ul>
        </nav>
    })
}

#[component]
async fn site_footer() -> Result<impl View> {
    Ok(view! {
        <footer class="footer"><p>"Slipway Marina · Built with Topcoat"</p></footer>
    })
}

#[component]
async fn status_badge(status: &Status) -> Result<impl View> {
    // TODO(lab-02) step 4: replace this with a `match status { ... }` inside
    // `view!`, giving each variant its own label. `Status::Occupied { vessel }`
    // should render "Occupied by " followed by the vessel name.
    let _ = status;

    Ok(view! { <span class="badge">"Unknown"</span> })
}

#[component]
async fn berth_list(berths: Vec<Berth>) -> Result<impl View> {
    // Goes away once the `for` loop below renders the real data.
    let _ = berths;

    // TODO(lab-02) step 4: wrap the list in `if berths.is_empty() { ... } else { ... }`
    // so an empty marina renders a message instead of an empty <ul>.
    Ok(view! {
        <ul class="berths">
            // TODO(lab-02) step 3: loop with `for berth in berths { ... }`.
            // TODO(lab-02) step 2: interpolate the berth's name and length with `(expr)`,
            // and its URL into the href with `href=(berth.url())`.
            // TODO(lab-02) step 7: give the <article> its attributes with
            // `attributes! { class="berth-card" data-berth=(berth.slug) ... }`
            // and spread them as `<article (attrs)>`.
            <li>
                <article class="berth-card">
                    <h2><a href="/berths/a1">"Berth A1"</a></h2>
                    <p class="berth-length">"8m"</p>
                    status_badge(status: &Status::Vacant)
                </article>
            </li>
        </ul>
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
                // TODO(lab-02) step 1: move the header, nav and footer markup here
                // if you started from a blank page; otherwise leave as-is.
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
