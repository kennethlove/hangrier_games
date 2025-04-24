use std::rc::Rc;
use dioxus::prelude::*;
#[derive(Clone, Debug, PartialEq, Props)]
pub struct InputProperties {
    r#type: String,
    name: String,
    placeholder: Option<String>,
    id: Option<String>,
    value: Option<String>,
    oninput: Option<EventHandler<Event<FormData>>>,
}

#[component]
pub fn Input(props: InputProperties) -> Element {
    let oninput = props.oninput.unwrap_or_default();
    rsx! {
        input {
            r#type: "{props.clone().r#type}",
            name: "{props.clone().name}",
            id: "{props.clone().id.unwrap_or_default()}",
            value: "{props.clone().value.unwrap_or_default()}",
            placeholder: "{props.clone().placeholder.unwrap_or_default()}",
            oninput: move |e: Event<FormData>| {
                if let Some(oninput) = props.oninput {
                    oninput.call(e);
                }
            },

            class: r#"
            block
            border
            w-half
            px-2
            py-1
            transition

            theme1:border-amber-600
            theme1:text-amber-200
            theme1:placeholder-amber-200/50
            theme1:bg-stone-800/65
            theme1:hover:bg-stone-800/75
            theme1:focus:bg-stone-800/75

            theme2:border-green-400
            theme2:text-green-200
            theme2:placeholder-green-200/50

            theme3:bg-stone-50/50
            theme3:border-yellow-600
            theme3:placeholder-stone-500
            theme3:text-stone-800
            "#,
        }
    }
}
