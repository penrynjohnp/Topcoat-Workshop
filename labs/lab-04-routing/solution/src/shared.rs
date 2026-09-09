use topcoat::{
    Result,
    view::{Attributes, Child, View, ViewExt, class, component, view},
};

pub const FEATURED: &str = "a1";
pub const NAV: [(&str, &str); 2] = [("/", "Home"), ("/berths", "Berths")];

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
