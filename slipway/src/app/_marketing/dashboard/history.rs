use crate::{
    auth::{AppState, require_auth},
    models::WorkOrder,
};
use std::time::Duration;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{page, request::uri},
    view::{View, component, emit, error_boundary, live, suspense, view},
};

// ANCHOR: live-progress
#[component]
async fn loading_progress() -> Result<impl View> {
    Ok(live! {
        emit! { <p data-progress="starting">"Opening the history log…"</p> }?;
        tokio::time::sleep(Duration::from_millis(10)).await;
        emit! { <p data-progress="checking">"Checking archived work orders…"</p> }?;
        tokio::time::sleep(Duration::from_millis(10)).await;
        emit! { <p data-progress="ready">"History log ready."</p> }
    })
}
// ANCHOR_END: live-progress

#[component]
async fn work_order_history(cx: &Cx, fail: bool) -> Result<impl View> {
    tokio::time::sleep(Duration::from_millis(20)).await;
    if fail {
        return Err(std::io::Error::other("history store unavailable").into());
    }
    let mut db = app_context::<AppState>(cx).db.clone();
    let orders = WorkOrder::filter(WorkOrder::fields().completed().eq(true))
        .include(WorkOrder::fields().berth())
        .exec(&mut db)
        .await?;
    Ok(view! {
        <ol data-component="work-order-history">
            for order in orders {
                <li>
                    (&order.title)
                    " · Berth "
                    (&order.berth.get().name)
                </li>
            }
        </ol>
    })
}

// ANCHOR: streamed-history
#[page]
async fn history(cx: &Cx) -> Result<impl View> {
    require_auth(cx).await?;
    let fail = uri(cx)
        .query()
        .is_some_and(|query| query.split('&').any(|pair| pair == "fail=true"));

    Ok(view! {
        <h1>"Work-order history"</h1>
        loading_progress()
        error_boundary(
            fallback: |error| Ok(
                    view! {
                        <p role="alert" data-history-error="true">
                            "History is unavailable: "
                            (error.to_string())
                        </p>
                    },
                ),
            suspense(
                fallback: view! { <p data-history-loading="true">"Loading history…"</p> },
                work_order_history(fail: fail)
            )
        )
    })
}
// ANCHOR_END: streamed-history
