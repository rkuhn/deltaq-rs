macro_rules! cloned {
    ($($name:ident),*; $e:expr) => {{
        $(let $name = $name.clone();)*
        $e
    }};
}

use deltaq_rs::{DeltaQ, DeltaQComponent, EvaluationContext, CDF};
use html::RenderResult;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, RequestInit};
use yew::{platform, prelude::*, suspense::use_future_with};

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
                        .text()?,
                )
                .await
            }
            Err(e) => Err(e),
        }
    })?;
    Ok(match &*json {
        Ok(cdf) => match serde_json::from_str::<T>(&cdf.as_string().unwrap()) {
            Ok(cdf) => Ok(cdf),
            Err(e) => Err(format!("{cdf:?} Deserialisation error: {}", e)),
        },
        Err(e) => Err(format!("Error: {e:?}")),
    })
}

async fn put_json<T: serde::Serialize>(url: &str, value: T) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let value = serde_json::to_string(&value).unwrap();
    let init = RequestInit::new();
    init.set_method("PUT");
    {
        let headers = web_sys::Headers::new().unwrap();
        headers.set("Content-Type", "application/json").unwrap();
        init.set_headers(&headers);
    }
    init.set_body(&value.into());
    JsFuture::from(
        JsFuture::from(window.fetch_with_str_and_init(url, &init))
            .await?
            .dyn_into::<web_sys::Response>()?
            .text()?,
    )
    .await
}

async fn delete_path(url: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let init = RequestInit::new();
    init.set_method("DELETE");
    JsFuture::from(
        JsFuture::from(window.fetch_with_str_and_init(url, &init))
            .await?
            .dyn_into::<web_sys::Response>()?
            .text()?,
    )
    .await
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
    let onclick = cloned!(selected; Callback::from(move |n| selected.set(Some(n))));

    // epoch counter to trigger recomputation when the context changes
    let epoch = use_state(|| 0);

    let cdf = use_json::<_, CDF>(
        (selected.clone(), epoch.clone()),
        cloned!(location; move |selected| {
            (*selected)
                .0
                .as_ref()
                .ok_or(JsValue::NULL)
                .map(|s| format!("{location}delta_q/{}", s))
        }),
    )?
    .map(|cdf| cdf.to_string())
    .unwrap_or_else(|e| e);

    let ctx = use_state(move || ctx);
    let update = cloned!(ctx, selected, epoch, location;
        Callback::from(move |dq: DeltaQ| {
            let Some(name) = (*selected).clone() else {
                return;
            };
            let mut cx = (*ctx).clone();
            cx.put(name.clone(), dq.clone());
            ctx.set(cx);
            platform::spawn_local(cloned!(epoch, location, name; async move {
                put_json(&format!("{location}delta_q/{name}"), dq).await.unwrap();
                epoch.set(*epoch + 1);
            }));
        })
    );

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
