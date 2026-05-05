use dioxus::prelude::*;
#[derive(Clone, Debug, PartialEq, Props)]
pub struct InputProperties {
    r#type: String,
    name: String,
    placeholder: Option<String>,
    id: Option<String>,
    value: Option<String>,
    oninput: Option<EventHandler<Event<FormData>>>,
    class: Option<String>,
}

#[component]
pub fn Input(props: InputProperties) -> Element {
    let classes = props.class.clone().unwrap_or_default();

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

            {classes}
            "#,
        }
    }
}
