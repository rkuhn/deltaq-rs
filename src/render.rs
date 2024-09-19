use crate::delta_q::DeltaQ;
use yew::prelude::*;

pub struct DeltaQComponent {
    delta_q: DeltaQ,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub delta_q: DeltaQ,
}

impl Component for DeltaQComponent {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            delta_q: ctx.props().delta_q.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _: Self::Message) -> bool {
        false
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.delta_q = ctx.props().delta_q.clone();
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>
                { self.render_delta_q(&self.delta_q) }
            </div>
        }
    }
}

impl DeltaQComponent {
    fn render_delta_q(&self, delta_q: &DeltaQ) -> Html {
        match delta_q {
            DeltaQ::Name(name) => html! { <span>{ name }</span> },
            DeltaQ::CDF(cdf) => html! { <span>{ format!("{}", cdf) }</span> },
            DeltaQ::Seq(first, second) => html! {
                <span>
                    { self.render_delta_q(first) }
                    { " •->-• " }
                    { self.render_delta_q(second) }
                </span>
            },
            DeltaQ::Choice(first, first_weight, second, second_weight) => html! {
                <span>
                    { self.render_delta_q(first) }
                    { format!(" {}⇌{} ", first_weight, second_weight) }
                    { self.render_delta_q(second) }
                </span>
            },
            DeltaQ::ForAll(first, second) => html! {
                <span>
                    { "∀(" }
                    { self.render_delta_q(first) }
                    { "|" }
                    { self.render_delta_q(second) }
                    { ")" }
                </span>
            },
            DeltaQ::ForSome(first, second) => html! {
                <span>
                    { "∃(" }
                    { self.render_delta_q(first) }
                    { "|" }
                    { self.render_delta_q(second) }
                    { ")" }
                </span>
            },
        }
    }
}
