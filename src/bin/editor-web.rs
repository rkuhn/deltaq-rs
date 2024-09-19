use deltaq_rs::{DeltaQ, DeltaQComponent};
use std::time::Duration;
use yew::{platform::time::sleep, prelude::*, suspense::use_future};

#[function_component]
fn DeQ() -> HtmlResult {
    let delta_q = use_future(|| async {
        sleep(Duration::from_secs(5)).await;
        DeltaQ::Name("Example".to_string())
    })?;
    Ok(html! { <DeltaQComponent delta_q={(*delta_q).clone()} /> })
}

#[function_component(App)]
fn app() -> Html {
    let waiting = html! { <p>{ "Waiting for DeltaQ..." }</p> };

    html! {
        <div>
            <h1>{ "DeltaQ Editor" }</h1>
            <Suspense fallback={waiting}>
                <DeQ />
            </Suspense>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
