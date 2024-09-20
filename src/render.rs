use crate::delta_q::DeltaQ;
use std::sync::Arc;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub delta_q: DeltaQ,
    pub on_change: Callback<DeltaQ>,
}

#[function_component(DeltaQComponent)]
pub fn delta_q_component(props: &Props) -> Html {
    match &props.delta_q {
        DeltaQ::Name(name) => {
            html! { <div style="border: 4px solid orange; margin: 4px; padding: 4px;">{ name }</div> }
        }
        DeltaQ::CDF(cdf) => {
            html! { <div style="border: 4px solid orange; margin: 4px; padding: 4px;">{ format!("{}", cdf) }</div> }
        }
        DeltaQ::Seq(first, second) => {
            html!(<Seq first={(**first).clone()} second={(**second).clone()} on_change={props.on_change.clone()} />)
        }
        DeltaQ::Choice(first, first_weight, second, second_weight) => {
            html!(<Branch top={(**first).clone()} bottom={(**second).clone()} on_change={props.on_change.clone()} kind={BranchKind::Choice(*first_weight, *second_weight)} />)
        }
        DeltaQ::ForAll(first, second) => {
            html!(<Branch top={(**first).clone()} bottom={(**second).clone()} kind={BranchKind::ForAll} on_change={props.on_change.clone()} />)
        }
        DeltaQ::ForSome(first, second) => {
            html!(<Branch top={(**first).clone()} bottom={(**second).clone()} kind={BranchKind::ForSome} on_change={props.on_change.clone()} />)
        }
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SeqProps {
    pub first: DeltaQ,
    pub second: DeltaQ,
    pub on_change: Callback<DeltaQ>,
}

#[function_component(Seq)]
pub fn seq(props: &SeqProps) -> Html {
    let on_change = props.on_change.clone();
    let first = props.first.clone();
    let second = props.second.clone();

    let on_first_change = Callback::from({
        let second = second.clone();
        let on_change = on_change.clone();
        move |delta_q| {
            on_change.emit(DeltaQ::Seq(Box::new(delta_q), Box::new(second.clone())));
        }
    });

    let on_second_change = Callback::from({
        let first = first.clone();
        move |delta_q| {
            on_change.emit(DeltaQ::Seq(Box::new(first.clone()), Box::new(delta_q)));
        }
    });

    html! {
        <div style="display: flex; flex-direction: row; align-items: center;">
            <DeltaQComponent delta_q={first} on_change={on_first_change} />
            <div style="width: 10px; height: 10px; border: 2px solid black; margin: 4px"></div>
            <DeltaQComponent delta_q={second} on_change={on_second_change} />
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct BranchProps {
    pub top: DeltaQ,
    pub bottom: DeltaQ,
    pub on_change: Callback<DeltaQ>,
    pub kind: BranchKind,
}

#[derive(Clone, Copy, PartialEq)]
pub enum BranchKind {
    Choice(f64, f64),
    ForAll,
    ForSome,
}

#[derive(Properties, Clone, PartialEq)]
pub struct BranchKindProps {
    pub kind: BranchKind,
}

#[function_component(BranchKindComponent)]
pub fn branch_kind_component(props: &BranchKindProps) -> Html {
    let kind = match &props.kind {
        BranchKind::Choice(first_weight, second_weight) => html! {
            <div style="display: flex; flex-direction: column; align-items: center;">
                <div>{first_weight}</div>
                <div>{"⇌"}</div>
                <div>{second_weight}</div>
            </div>
        },
        BranchKind::ForAll => html! { <div>{ "∀" }</div> },
        BranchKind::ForSome => html! { <div>{ "∃" }</div> },
    };
    html!(<div style="display: flex; align-items: center; justify-content: center; padding: 8px;">{ kind }</div>)
}

/// A component that renders a branch of a DeltaQ tree.
///
/// The HTML representation consists of two DIV, with the left showing the branch kind and the right showing the branch content.
/// The branch content is rendered in two DIV, the top branch at the top and the bottom branch at the bottom.
/// There is a border between the two branches and to the right of the branch kind.
#[function_component(Branch)]
fn branch(props: &BranchProps) -> Html {
    let on_change = props.on_change.clone();
    let top = props.top.clone();
    let bottom = props.bottom.clone();
    let kind = props.kind;
    let constructor: Arc<dyn Fn(Box<DeltaQ>, Box<DeltaQ>) -> DeltaQ> = match kind {
        BranchKind::Choice(l, r) => Arc::new(move |dql, dqr| DeltaQ::Choice(dql, l, dqr, r)),
        BranchKind::ForAll => Arc::new(DeltaQ::ForAll),
        BranchKind::ForSome => Arc::new(DeltaQ::ForSome),
    };

    let on_top_change = Callback::from({
        let bottom = bottom.clone();
        let on_change = on_change.clone();
        let constructor = constructor.clone();
        move |delta_q| {
            on_change.emit(constructor(Box::new(delta_q), Box::new(bottom.clone())));
        }
    });

    let on_bottom_change = Callback::from({
        let top = top.clone();
        move |delta_q| {
            on_change.emit(constructor(Box::new(top.clone()), Box::new(delta_q)));
        }
    });

    html! {
        <div style="display: flex; flex-direction: row; margin: 4px; border: 1px solid grey;">
            <BranchKindComponent kind={kind} />
            <div style="display: flex; flex-direction: column; align-items: left; border-left: 2px solid black;">
                <div style="display: flex; flex-direction: row; align-items: left;">
                    <DeltaQComponent delta_q={top} on_change={on_top_change} />
                </div>
                <div style="border: 1px solid black;"></div>
                <div style="display: flex; flex-direction: row; align-items: left;">
                    <DeltaQComponent delta_q={bottom} on_change={on_bottom_change} />
                </div>
            </div>
        </div>
    }
}
