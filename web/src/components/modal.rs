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

                            theme1:bg-stone-200
                            theme1:text-stone-900

                            theme2:text-green-900
                            theme2:bg-green-200
                            theme2:rounded-md
                            theme2:p-2

                            theme3:bg-stone-50
                            theme3:border-5
                            theme3:border-gold-rich
                            theme3:p-5
                            theme3:pb-2
                            "#,

                            h1 {
                                class: r#"
                                block
                                text-lg

                                theme1:bg-red-900
                                theme1:text-stone-200
                                theme1:font-[Cinzel]
                                theme1:p-4

                                theme2:bg-green-800
                                theme2:text-green-200
                                theme2:font-[Playfair_Display]
                                theme2:rounded-md
                                theme2:p-2

                                theme3:font-[Orbitron]
                                theme3:text-3xl
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
