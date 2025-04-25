use dioxus::prelude::*;
use crate::components::icons::loading::LoadingIcon;
use crate::components::icons::mockingjay::Mockingjay;
use crate::components::icons::mockingjay_arrow::MockingjayArrow;
use crate::LoadingState;

#[component]
pub fn LoadingModal() -> Element {
    let loading_signal = use_context::<Signal<LoadingState>>();

    let open = match *loading_signal.read() {
        LoadingState::Loading => true,
        _ => false,
    };

    rsx! {
        dialog {
            open: open,
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
                            p-4
                            grid
                            grid-col
                            gap-4

                            theme1:bg-stone-200
                            theme1:text-stone-900

                            theme2:text-green-900
                            theme2:bg-green-200

                            theme3:bg-stone-50
                            theme3:border-3
                            theme3:border-gold-rich
                            "#,

                            h1 {
                                class: r#"
                                block
                                p-2
                                text-lg
                                theme1:bg-red-900
                                theme1:text-stone-200
                                theme1:font-[Cinzel]

                                theme2:bg-green-800
                                theme2:text-green-200
                                theme2:font-[Playfair_Display]

                                theme3:font-[Orbitron]
                                "#,

                                "Loading..."
                            }
                            div {
                                class: "flex justify-center",
                                MockingjayArrow {
                                    class: r#"
                                    size-16
                                    animate-spin
                                    theme1:fill-red-900
                                    theme2:fill-green-800
                                    theme3:fill-amber-600
                                    "#
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
