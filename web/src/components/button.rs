use dioxus::prelude::*;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct ButtonProps {
    pub extra_classes: Option<String>,
    pub title: Option<String>,
    pub onclick: Option<EventHandler<Rc<MouseData>>>,
    pub children: Option<Element>,
    pub r#type: Option<String>,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let onclick = props.onclick.unwrap_or_default();
    let r#type = props.r#type.unwrap_or_else(|| "button".to_string());
    let extra_classes = props.extra_classes.unwrap_or_default();

    rsx! {
        button {
            class: "button border px-2 py-1 cursor-pointer {extra_classes}",
            r#type,
            onclick: move |event| { onclick.call(event.data()) },
            title: props.title.unwrap_or_default(),
            {props.children}
        }
    }
}
