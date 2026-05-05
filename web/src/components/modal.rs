use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct Props {
    pub title: String,
    pub open: bool,
    pub children: Option<Element>,
}

#[component]
pub fn Modal(modal_props: Props) -> Element {
    rsx! {
        dialog {
            open: modal_props.open,
            div {
                class: r#"
                fixed
                inset-0
                backdrop-blur-sm
                backdrop-grayscale
                "#,

                div {
                    class: "fixed inset-0 z-10 w-screen h-screen overflow-y-hidden",
                    div {
                        class: r#"
                        flex
                        flex-col
                        min-h-full
                        items-center
                        justify-center
                        "#,

                        div {
                            class: r#"
                            grid
                            grid-col
                            gap-4

                            "#,

                            h1 {
                                class: r#"
                                block
                                text-lg

                                "#,

                                {modal_props.title}
                            }
                            div {
                                class: "flex justify-center pb-4",
                                {modal_props.children},
                            }
                        }
                    }
                }
            }
        }
    }
}
