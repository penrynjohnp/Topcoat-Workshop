use crate::{
    auth::{AppState, require_auth},
    models::Berth,
    shared::managed_berths,
};
use serde::Deserialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{HeaderValue, StatusCode, content::Form, header, page},
    view::{View, component, view},
};

const MAX_TITLE_LEN: usize = 80;

#[derive(Clone, Default, Deserialize)]
struct WorkOrderInput {
    title: String,
    berth_slug: String,
}

#[derive(Default)]
struct ValidationErrors {
    title: Option<&'static str>,
    berth: Option<&'static str>,
}

impl ValidationErrors {
    fn any(&self) -> bool {
        self.title.is_some() || self.berth.is_some()
    }
}

// ANCHOR: work-order-form
#[component]
async fn work_order_form(
    input: &WorkOrderInput,
    errors: &ValidationErrors,
    berths: &Vec<Berth>,
) -> Result<impl View> {
    let submitted_berth_is_listed = berths.iter().any(|berth| berth.slug == input.berth_slug);
    Ok(view! {
        <h1>"Create work order"</h1>
        <form method="post" action="/dashboard/work-orders/new" novalidate="novalidate">
            <label for="work-order-title">"Title"</label>
            <input
                id="work-order-title"
                name="title"
                value=(&input.title)
                aria-invalid=(errors.title.is_some().then_some("true"))
            >
            if let Some(error) = errors.title {
                <p class="error" data-error="title">(error)</p>
            }

            <label for="work-order-berth">"Berth"</label>
            <select
                id="work-order-berth"
                name="berth_slug"
                aria-invalid=(errors.berth.is_some().then_some("true"))
            >
                <option value="">"Choose a berth"</option>
                if !input.berth_slug.is_empty() {
                    if !submitted_berth_is_listed {
                        <option value=(&input.berth_slug) selected="selected">
                            "Unavailable berth: "
                            (&input.berth_slug)
                        </option>
                    }
                }
                for berth in berths {
                    <option
                        value=(&berth.slug)
                        selected=((berth.slug == input.berth_slug).then_some("selected"))
                    >
                        (&berth.name)
                    </option>
                }
            </select>
            if let Some(error) = errors.berth {
                <p class="error" data-error="berth">(error)</p>
            }

            <button type="submit">"Create work order"</button>
        </form>
    })
}
// ANCHOR_END: work-order-form

#[page(GET)]
async fn form(cx: &Cx) -> Result<impl View> {
    require_auth(cx).await?;
    let berths = managed_berths(cx).await?;
    let input = WorkOrderInput::default();
    let errors = ValidationErrors::default();
    Ok(view! { work_order_form(input: &input, errors: &errors, berths: &berths) })
}

// ANCHOR: manual-form-validation
// TODO(lab-10): Validate both fields, preserve invalid values, and create via the relation.
#[page(POST)]
async fn create(cx: &Cx, Form(mut input): Form<WorkOrderInput>) -> Result<impl View> {
    require_auth(cx).await?;
    input.title = input.title.trim().to_owned();
    let berths = managed_berths(cx).await?;

    let mut errors = ValidationErrors::default();
    if input.title.is_empty() {
        errors.title = Some("Enter a title.");
    } else if input.title.len() > MAX_TITLE_LEN {
        errors.title = Some("Keep the title to 80 characters or fewer.");
    }
    let berth = berths.iter().find(|berth| berth.slug == input.berth_slug);
    if berth.is_none() {
        errors.berth = Some("Choose a managed berth.");
    }

    let created = !errors.any();
    if let Some(berth) = berth.filter(|_| created) {
        let mut db = app_context::<AppState>(cx).db.clone();
        toasty::create!(in berth.work_orders() {
            title: input.title.as_str(),
            completed: false,
        })
        .exec(&mut db)
        .await?;
    }

    let redirect_header = (header::LOCATION, HeaderValue::from_static("/dashboard"));
    Ok(view! {
        if created {
            (StatusCode::SEE_OTHER)
            (redirect_header)
        } else {
            work_order_form(input: &input, errors: &errors, berths: &berths)
        }
    })
}
// ANCHOR_END: manual-form-validation
