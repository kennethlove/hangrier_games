use dioxus::dioxus_core::AttributeValue;
use dioxus::prelude::*;
use crate::components::icons::mockingjay::Mockingjay;
use crate::components::icons::mockingjay_arrow::MockingjayArrow;
use crate::components::icons::mockingjay_flight::MockingjayFlight;
use crate::components::icons::three_finger_salute::ThreeFingerSalute;
use crate::routes::Routes;
use crate::storage::{use_persistent, AppState, Colorscheme};

#[component]
pub fn Navbar() -> Element {
    let mut storage = use_persistent("hangry-games", || AppState::default());
    let mut theme_signal: Signal<Colorscheme> = use_context();

    rsx! {
        header {
            class: "flex flex-col flex-wrap items-center",
            h1 {
                class: r#"
                text-5xl
                sm:text-6xl
                cinzel-font
                theme1:text-transparent
                font-bold
                bg-clip-text
                bg-radial
                theme1:from-amber-300
                theme1:to-red-600
                "#,
                Link { to: Routes::Home {}, "Hangry Games" }
            }

            nav {
                class: "cinzel-font text-xl theme1:text-amber-500",
                ul {
                    class: "flex flex-row gap-16",
                    li {
                        class: "px-2 hover:bg-amber-500 theme2:hover:text-green-900 theme1:hover:text-red-900 theme3:hover:text-blue-900",
                        Link { to: Routes::Home {}, "Home" }
                    }
                    li {
                        class: "px-2 hover:bg-amber-500 theme2:hover:text-green-900 theme1:hover:text-red-900 theme3:hover:text-blue-900",
                        Link { to: Routes::GamesList {}, "Games" }
                    }
                    li {
                        class: "relative group inline-block",
                        input {
                            id: "theme-switcher",
                            r#type: "checkbox",
                            class: "peer sr-only"
                        }
                        label {
                            class: r#"px-2
                                border border-transparent
                                theme1:peer-focus:bg-amber-500
                                theme1:peer-checked:bg-amber-500
                                theme1:group-hover:bg-amber-500
                                theme1:group-hover:text-red-900
                                theme1:peer-focus:text-red-900
                                theme1:peer-checked:text-red-900
                            "#,
                            r#for: "theme-switcher",
                            "Theme",
                        }
                        div {
                            class: r#"absolute
                                right-0
                                sm:left-0
                                z-99
                                opacity-0
                                w-64
                                invisible
                                group-hover:opacity-100
                                group-hover:visible
                                bg-linear-to-b
                                theme1:from-amber-500
                                theme1:to-amber-700
                                peer-checked:opacity-100
                                peer-checked:visible
                                peer-focus:opacity-100
                                peer-focus:visible
                            "#,

                            div {
                                class: "grid grid-cols-3 place-content-center gap-4 p-4",
                                button {
                                    class: "button size-16 cursor-pointer",
                                    onclick: move |_| {
                                        let mut state = storage.get();
                                        state.to_theme_one();
                                        storage.set(state.clone());
                                        theme_signal.set(state.colorscheme)
                                    },
                                    MockingjayArrow { class: "fill-red-900 theme1:stroke-amber-200 hover:stroke-red-200 stroke-50" }
                                }
                                button {
                                    class: "button size-16 cursor-pointer",
                                    onclick: move |_| {
                                        let mut state = storage.get();
                                        state.to_theme_two();
                                        storage.set(state.clone());
                                        theme_signal.set(state.colorscheme)
                                    },
                                    Mockingjay { class: "fill-green-900 theme2:stroke-amber-200 hover:stroke-green-200 stroke-50" }
                                }
                                button {
                                    class: "button size-16 cursor-pointer",
                                    onclick: move |_| {
                                        let mut state = storage.get();
                                        state.to_theme_three();
                                        storage.set(state.clone());
                                        theme_signal.set(state.colorscheme)
                                    },
                                    MockingjayFlight {class: "fill-blue-900 theme3:stroke-amber-200 hover:stroke-blue-200 stroke-50" }
                                }
                            }
                        }
                    }
                }
            }
        }
        Outlet::<Routes> {}
    }
}

