use crate::auth::{AppState, current_user_uncached, require_auth};
use std::time::Duration;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{page, request::uri},
    runtime::{connected, shard},
    view::{SuspenseMode, View, component, emit, error_boundary, live, suspense, view},
};

// ANCHOR: connected-work-order-status
#[shard("/dashboard/work-orders/status")]
async fn work_order_status(cx: &Cx) -> Result<impl View> {
    require_auth(cx).await?;
    Ok(live! {
        let state: &AppState = app_context(cx);
        let mut updates = state.updates.subscribe();
        loop {
            if current_user_uncached(cx).await.is_none() {
                return emit! { <p role="alert">"Your session has expired."</p> };
            }
            let completed = state.completed_work_orders.lock().unwrap().len();
            let token = emit! {
                <section data-component="live-work-order-status">
                    <h2>"Work-order status"</h2>
                    <p data-completed-count=(completed)>
                        "Completed work orders: "
                        (completed)
                    </p>
                </section>
            }?;
            if !connected(cx) {
                return Ok(token);
            }
            if matches!(
                updates.recv().await,
                Err(tokio::sync::broadcast::error::RecvError::Closed),
            ) {
                return Ok(token);
            }
        }
    })
}
// ANCHOR_END: connected-work-order-status

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
async fn work_order_history(fail: bool) -> Result<impl View> {
    tokio::time::sleep(Duration::from_millis(20)).await;
    if fail {
        return Err(std::io::Error::other("history store unavailable").into());
    }
    Ok(view! {
        <ol data-component="work-order-history">
            <li>"Lady Jane lifted · 4 September"</li>
            <li>"C3 mooring inspected · 7 September"</li>
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
    let mode = if uri(cx)
        .query()
        .is_some_and(|query| query.split('&').any(|pair| pair == "mode=wait"))
    {
        SuspenseMode::Wait
    } else {
        SuspenseMode::Stream
    };

    Ok(view! {
        <h1>"Work-order history"</h1>
        <form action="/dashboard/history" method="get">
            <label for="history-mode">"History display"</label>
            <select id="history-mode" name="mode">
                <option value="stream" selected=(mode == SuspenseMode::Stream)>
                    "Stream"
                </option>
                <option value="wait" selected=(mode == SuspenseMode::Wait)>
                    "Wait"
                </option>
            </select>
            <button type="submit">"Apply"</button>
        </form>
        loading_progress()
        work_order_status()
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
                mode: mode,
                fallback: view! { <p data-history-loading="true">"Loading history…"</p> },
                work_order_history(fail: fail)
            )
        )
    })
}
// ANCHOR_END: streamed-history
