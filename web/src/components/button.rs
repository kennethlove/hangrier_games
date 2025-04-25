use dioxus::prelude::*;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct ButtonProps {
    pub class: Option<String>,
    pub title: Option<String>,
    pub onclick: Option<EventHandler<Rc<MouseData>>>,
    pub children: Option<Element>,
    pub r#type: Option<String>,
    pub disabled: Option<bool>
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

    let classes = format!(r#"
    theme1:bg-radial
    theme1:from-amber-300
    theme1:to-red-500
    theme1:border-red-500
    theme1:text-red-900
    theme1:hover:text-stone-200
    theme1:hover:from-amber-500
    theme1:hover:to-red-700
    theme1:rounded-sm

    theme2:text-green-800
    theme2:bg-linear-to-b
    theme2:from-green-400
    theme2:to-teal-500
    theme2:border-none
    theme2:hover:text-green-200
    theme2:hover:from-green-500
    theme2:hover:to-teal-600
    theme2:rounded-md

    theme3:border-none
    theme3:bg-gold-rich
    theme3:hover:bg-gold-rich-reverse
    theme3:text-stone-700
    theme3:hover:text-stone-50

    {extra_classes}
    "#);

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
