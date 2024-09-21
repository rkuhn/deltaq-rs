use crate::delta_q::DeltaQ;
use std::sync::Arc;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub delta_q: DeltaQ,
    pub on_change: Callback<DeltaQ>,
}

#[function_component(DeltaQComponent)]
pub fn delta_q_component(props: &Props) -> Html {
    let on_change = props.on_change.clone();
    match &props.delta_q {
        DeltaQ::BlackBox => {
            html! { <BlackBox {on_change} /> }
        }
        DeltaQ::Name(name) => {
            html! { <NameComponent name={name.clone()} {on_change} /> }
        }
        DeltaQ::CDF(cdf) => {
            html! { <div class={classes!("cdf")}>{ format!("{}", cdf) }</div> }
        }
        DeltaQ::Seq(first, second) => {
            html!(<Seq first={(**first).clone()} second={(**second).clone()} {on_change} />)
        }
        DeltaQ::Choice(first, first_weight, second, second_weight) => {
            html!(<Branch top={(**first).clone()} bottom={(**second).clone()} {on_change} kind={BranchKind::Choice(*first_weight, *second_weight)} />)
        }
        DeltaQ::ForAll(first, second) => {
            html!(<Branch top={(**first).clone()} bottom={(**second).clone()} kind={BranchKind::ForAll} {on_change} />)
        }
        DeltaQ::ForSome(first, second) => {
            html!(<Branch top={(**first).clone()} bottom={(**second).clone()} kind={BranchKind::ForSome} {on_change} />)
        }
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct BlackBoxProps {
    pub on_change: Callback<DeltaQ>,
}

#[function_component(BlackBox)]
pub fn black_box(props: &BlackBoxProps) -> Html {
    let on_change = props.on_change.clone();
    let popup = use_state(|| false);
    let name = use_state(|| "".to_string());

    html! {
        <div class={classes!("blackBox", "anchor")} onclick={cloned!(popup; move |_| if !*popup { popup.set(true) })}>
            if *popup {
                <div class={classes!("popup")}>
                    <button onclick={cloned!(popup; move |_| popup.set(false))}>{ "abort" }</button>
                    <span>
                        <button onclick={cloned!(on_change, name; move |_| on_change.emit(DeltaQ::name(&name)))}>{ "refine" }</button>
                        <input type="text" value={(*name).clone()} onchange={cloned!(name;
                            move |e: Event| name.set(e.target_unchecked_into::<HtmlInputElement>().value()))} />
                    </span>
                </div>
            }
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct NameProps {
    pub name: String,
    pub on_change: Callback<DeltaQ>,
}

#[function_component(NameComponent)]
pub fn name_component(props: &NameProps) -> Html {
    let on_change = props.on_change.clone();
    let popup = use_state(|| false);
    let name = use_state(|| props.name.clone());

    html! {
        <div class={classes!("name", "anchor")} onclick={cloned!(popup; move |_| if !*popup { popup.set(true) })}>
            { &*name }
            if *popup {
                <div class={classes!("popup")}>
                    <button onclick={cloned!(popup; move |_| popup.set(false))}>{ "abort" }</button>
                    <span>
                        <button onclick={cloned!(on_change, name, popup; move |_| { popup.set(false); on_change.emit(DeltaQ::Name((*name).clone())) })}>{ "change" }</button>
                        <input type="text" value={(*name).clone()} onchange={cloned!(name;
                            move |e: Event| name.set(e.target_unchecked_into::<HtmlInputElement>().value()))} />
                    </span>
                    <button onclick={cloned!(on_change, name; move |_| on_change.emit(DeltaQ::seq(DeltaQ::name(&name), DeltaQ::BlackBox)))}>{ "append" }</button>
                    <button onclick={cloned!(on_change, name; move |_| on_change.emit(DeltaQ::choice(DeltaQ::name(&name), 1.0, DeltaQ::BlackBox, 1.0)))}>{ "make choice" }</button>
                    <button onclick={cloned!(on_change, name; move |_| on_change.emit(DeltaQ::for_all(DeltaQ::name(&name), DeltaQ::BlackBox)))}>{ "make forAll" }</button>
                    <button onclick={cloned!(on_change, name; move |_| on_change.emit(DeltaQ::for_some(DeltaQ::name(&name), DeltaQ::BlackBox)))}>{ "make forSome" }</button>
                    <button onclick={cloned!(on_change; move |_| on_change.emit(DeltaQ::BlackBox))}>{ "black box" }</button>
                </div>
            }
        </div>
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

    let on_first_change = Callback::from(cloned!(second, on_change;
        move |delta_q| {
            on_change.emit(DeltaQ::Seq(Box::new(delta_q), Box::new(second.clone())));
        }
    ));

    let on_second_change = Callback::from(cloned!(first, on_change;
        move |delta_q| {
            on_change.emit(DeltaQ::Seq(Box::new(first.clone()), Box::new(delta_q)));
        }
    ));

    let popup = use_state(|| false);

    html! {
        <div class={classes!("row", "center", "frame")}>
            <DeltaQComponent delta_q={first.clone()} on_change={on_first_change} />
            <div class={classes!("seqSymbol", "anchor")} onclick={cloned!(popup; move |_| if !*popup { popup.set(true) })}>
                if *popup {
                    <div class={classes!("popup")}>
                    <button onclick={cloned!(popup; move |_| popup.set(false))}> { "abort" } </button>
                    <button onclick={cloned!(on_change, first, second; move |_| on_change.emit(DeltaQ::choice(DeltaQ::seq(first.clone(), second.clone()), 1.0, DeltaQ::BlackBox, 1.0)))}> { "make choice" } </button>
                    <button onclick={cloned!(on_change, first, second; move |_| on_change.emit(DeltaQ::for_all(DeltaQ::seq(first.clone(), second.clone()), DeltaQ::BlackBox)))}> { "make forAll" } </button>
                    <button onclick={cloned!(on_change, first, second; move |_| on_change.emit(DeltaQ::for_some(DeltaQ::seq(first.clone(), second.clone()), DeltaQ::BlackBox)))}> { "make forSome" } </button>
                    <button onclick={cloned!(on_change, first, second, popup; move |_| { popup.set(false); on_change.emit(DeltaQ::seq(second.clone(), first.clone())) })}>{ "switch" }</button>
                    <button onclick={cloned!(popup, on_change, first; move |_| { popup.set(false); on_change.emit(first.clone()) })}>{ "keep left" }</button>
                    <button onclick={cloned!(popup, on_change, second; move |_| { popup.set(false); on_change.emit(second.clone()) })}>{ "keep right" }</button>
                    </div>
                }
            </div>
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

#[function_component(BranchKindComponent)]
pub fn branch_kind_component(props: &BranchProps) -> Html {
    let kind = match &props.kind {
        BranchKind::Choice(first_weight, second_weight) => html! {
            <div class={classes!("column", "center")}>
                <div>{first_weight}</div>
                <div>{"⇌"}</div>
                <div>{second_weight}</div>
            </div>
        },
        BranchKind::ForAll => html! { <div>{ "∀" }</div> },
        BranchKind::ForSome => html! { <div>{ "∃" }</div> },
    };

    let popup = use_state(|| false);
    let on_change = props.on_change.clone();
    let top = props.top.clone();
    let bottom = props.bottom.clone();

    let top_frac = use_state(|| {
        if let BranchKind::Choice(l, _) = props.kind {
            l
        } else {
            1.0
        }
    });
    let bottom_frac = use_state(|| {
        if let BranchKind::Choice(_, r) = props.kind {
            r
        } else {
            1.0
        }
    });

    html!(
    <div class={classes!("row", "center", "branchKind", "anchor")} onclick={cloned!(popup; move |_| if !*popup { popup.set(true) })}>
        { kind }
        if *popup {
            <div class={classes!("popup")}>
                <button onclick={cloned!(popup; move |_| popup.set(false))}>{ "abort" }</button>
                <span>
                    <button onclick={cloned!(popup, on_change, top, bottom, top_frac, bottom_frac; move |_| {
                        popup.set(false);
                        on_change.emit(DeltaQ::choice(top.clone(), *top_frac, bottom.clone(), *bottom_frac))
                    })}>{ "make choice" }</button>
                    <input type="number" value={top_frac.to_string()} onchange={cloned!(top_frac;
                        move |e: Event| top_frac.set(e.target_unchecked_into::<HtmlInputElement>().value_as_number() as f64))} />
                    <input type="number" value={bottom_frac.to_string()} onchange={cloned!(bottom_frac;
                        move |e: Event| bottom_frac.set(e.target_unchecked_into::<HtmlInputElement>().value_as_number() as f64))} />
                </span>
                <button onclick={cloned!(popup, on_change, top, bottom; move |_| {
                    popup.set(false);
                    on_change.emit(DeltaQ::for_all(top.clone(), bottom.clone()))
                })}>{ "make forAll" }</button>
                <button onclick={cloned!(popup, on_change, top, bottom; move |_| {
                    popup.set(false);
                    on_change.emit(DeltaQ::for_some(top.clone(), bottom.clone()))
                })}>{ "make forSome" }</button>
                <button onclick={cloned!(popup, on_change, top, bottom; move |_| {
                    popup.set(false);
                    on_change.emit(DeltaQ::choice(bottom.clone(), *bottom_frac, top.clone(), *top_frac))
                })}>{ "switch" }</button>
                <button onclick={cloned!(popup, on_change, top; move |_| { popup.set(false); on_change.emit(top.clone()) })}>{ "keep top" }</button>
                <button onclick={cloned!(popup, on_change, bottom; move |_| { popup.set(false); on_change.emit(bottom.clone()) })}>{ "keep bottom" }</button>
                <button onclick={cloned!(on_change; move |_| on_change.emit(DeltaQ::BlackBox))}>{ "black box" }</button>
            </div>
        }
    </div>)
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

    let on_top_change = Callback::from(cloned!(bottom, on_change, constructor;
        move |delta_q| {
            on_change.emit(constructor(Box::new(delta_q), Box::new(bottom.clone())));
        }
    ));

    let on_bottom_change = Callback::from(cloned!(top, on_change;
        move |delta_q| {
            on_change.emit(constructor(Box::new(top.clone()), Box::new(delta_q)));
        }
    ));

    html! {
        <div class={classes!("row", "frame")}>
            <BranchKindComponent ..props.clone() />
            <div class={classes!("column", "left")} style="border-left: 2px solid black;">
                <div class={classes!("row", "left")} >
                    <DeltaQComponent delta_q={top} on_change={on_top_change} />
                </div>
                <div style="border: 1px solid black;"></div>
                <div class={classes!("row", "left")} >
                    <DeltaQComponent delta_q={bottom} on_change={on_bottom_change} />
                </div>
            </div>
        </div>
    }
}
