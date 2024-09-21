use deltaq_rs::{DeltaQComponent, EvaluationContext, CDF};
use gloo_utils::format::JsValueSerdeExt;
use html::RenderResult;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;
use yew::suspense::use_future_with;

#[hook]
fn use_json<D: PartialEq + 'static, T: for<'a> serde::Deserialize<'a>>(
    dep: D,
    url: impl Fn(Rc<D>) -> Result<String, JsValue> + 'static,
) -> RenderResult<Result<T, String>> {
    let window = web_sys::window().unwrap();
    let json = use_future_with(dep, move |dep| async move {
        match url(dep) {
            Ok(url) => {
                JsFuture::from(
                    JsFuture::from(window.fetch_with_str(&url))
                        .await?
                        .dyn_into::<web_sys::Response>()?
                        .json()?,
                )
                .await
            }
            Err(e) => Err(e),
        }
    })?;
    Ok(match &*json {
        Ok(cdf) => match cdf.into_serde::<T>() {
            Ok(cdf) => Ok(cdf),
            Err(e) => Err(format!("Deserialisation error: {}", e)),
        },
        Err(e) => Err(format!("Error: {e:?}")),
    })
}

#[function_component(AppMain)]
fn app_main() -> HtmlResult {
    let location = web_sys::window().unwrap().location().href().unwrap();
    let location2 = location.clone();

    let ctx =
        match use_json::<_, EvaluationContext>((), move |_| Ok(format!("{location2}delta_q")))? {
            Ok(ctx) => ctx,
            Err(e) => return Ok(html! { <p>{ e }</p> }),
        };

    let selected = use_state(|| Some("out".to_owned()));
    let onclick = {
        let selected = selected.clone();
        Callback::from(move |n| selected.set(Some(n)))
    };

    let cdf = use_json::<_, CDF>(selected.clone(), move |selected| {
        (**selected)
            .as_ref()
            .ok_or(JsValue::NULL)
            .map(|s| format!("{location}delta_q/{}", s))
    })?
    .map(|cdf| cdf.to_string())
    .unwrap_or_else(|e| e);

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
