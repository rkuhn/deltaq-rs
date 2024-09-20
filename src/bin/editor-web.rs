use deltaq_rs::{DeltaQComponent, EvaluationContext, CDF};
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;
use yew::suspense::{use_future, use_future_with};

#[function_component(AppMain)]
fn app_main() -> HtmlResult {
    let window = web_sys::window().unwrap();
    let location = window.location().href().unwrap();
    let data = use_future({
        let window = window.clone();
        let location = location.clone();
        move || async move {
            JsFuture::from(
                JsFuture::from(window.fetch_with_str(&format!("{location}delta_q")))
                    .await?
                    .dyn_into::<web_sys::Response>()?
                    .json()?,
            )
            .await
        }
    })?;

    let selected = use_state(|| None);
    let onclick = {
        let selected = selected.clone();
        Callback::from(move |n| selected.set(Some(n)))
    };

    let cdf_json = use_future_with(selected.clone(), move |selected| async move {
        if let Some(name) = &**selected {
            JsFuture::from(
                JsFuture::from(window.fetch_with_str(&format!("{location}delta_q/{}", *name)))
                    .await?
                    .dyn_into::<web_sys::Response>()?
                    .json()?,
            )
            .await
        } else {
            Ok(JsValue::NULL)
        }
    })?;
    let cdf = match &*cdf_json {
        Ok(cdf) => match cdf.into_serde::<CDF>() {
            Ok(cdf) => cdf.to_string(),
            Err(e) => format!("Deserialisation error: {}", e),
        },
        Err(e) => format!("Error: {e:?}"),
    };

    let ctx = match &*data {
        Ok(dq) => match dq.into_serde::<EvaluationContext>() {
            Ok(dq) => dq,
            Err(e) => return Ok(html! { <p>{ format!("Deserialisation error: {}", e) }</p> }),
        },

        Err(e) => return Ok(html! { <p>{ format!("Error: {e:?}") }</p> }),
    };

    let ctx = use_state(move || ctx);
    let update = {
        let ctx = ctx.clone();
        let selected = selected.clone();
        Callback::from(move |dq| {
            let Some(name) = (*selected).clone() else {
                return;
            };
            let mut cx = (*ctx).clone();
            cx.put(name, dq);
            ctx.set(cx);
        })
    };

    let mut sel_found = false;
    let list_items = ctx
        .iter()
        .map(|(k, v)| {
            let name = k.clone();
            let onclick = onclick.clone();
            let mut h = html! {
                <li onclick={onclick.reform(move |_| name.clone())}>
                    { format!("{k}: {v}") }
                </li>
            };
            if selected.as_ref() == Some(k) {
                sel_found = true;
                h = html! { <strong>{ h }</strong> };
            }
            h
        })
        .collect::<Html>();
    if selected.is_some() && !sel_found {
        selected.set(None);
    }

    Ok(html! {
    <div>
        <p>{ "context:" }</p>
        <ul>
        { list_items }
        </ul>
        if let Some(name) = selected.as_ref() {
            <p>{ "selected: " } { name }</p>
            <div style="background-color: #f0f0f0; padding: 4px; margin: 4px; display: flex; flex-direction: row;">
                <DeltaQComponent delta_q={ctx.get(name).unwrap().clone()} on_change={update} />
            </div>
            <p>{ "result CDF: " } { cdf }</p>
        }
    </div>
    })
}

#[function_component(App)]
fn app() -> Html {
    let waiting = html! { <p>{ "Waiting for DeltaQ..." }</p> };

    html! {
        <div>
            <h1>{ "DeltaQ Editor" }</h1>
            <Suspense fallback={waiting}>
                <AppMain />
            </Suspense>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
