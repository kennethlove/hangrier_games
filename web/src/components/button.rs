use dioxus::prelude::*;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct ButtonProps {
    pub class: Option<String>,
    pub title: Option<String>,
    pub onclick: Option<EventHandler<Rc<MouseData>>>,
    pub children: Option<Element>,
    pub r#type: Option<String>,
    pub disabled: Option<bool>,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let onclick = props.onclick.unwrap_or_default();
    let r#type = props.r#type.unwrap_or_else(|| "button".to_string());
    let extra_classes = props.class.unwrap_or_default();
    let is_disabled = props.disabled.unwrap_or(false); // Calculate disabled state

    rsx! {
        button {
            class: r#"
            button
            border
            px-2
            py-1
            cursor-pointer
            {extra_classes}
            "#,
            r#type,
            onclick: move |event| { onclick.call(event.data()) },
            title: props.title.unwrap_or_default(),
            // Use the boolean value directly for the disabled attribute
            disabled: is_disabled,
            {props.children}
        }
    }
}

#[component]
pub fn ThemedButton(props: ButtonProps) -> Element {
    let title = props.title.unwrap_or_default();
    let onclick = props.onclick.unwrap_or_default();
    let r#type = props.r#type.unwrap_or_else(|| "button".to_string());
    let extra_classes = props.class.unwrap_or_default();
    let is_disabled = props.disabled.unwrap_or(false); // Calculate disabled state

    let classes = format!(
        r#"

    {extra_classes}
    "#
    );

    rsx! {
        Button {
            class: classes,
            onclick,
            r#type,
            title,
            // Use the boolean value directly for the disabled attribute
            disabled: is_disabled,
            {props.children}
        }
    }
}
