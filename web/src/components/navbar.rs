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
                font-bold

                theme1:bg-radial
                theme1:bg-clip-text
                theme1:text-transparent
                theme1:from-amber-300
                theme1:to-red-600

                theme2:text-transparent
                theme2:bg-linear-to-b
                theme2:bg-clip-text
                theme2:from-teal-500
                theme2:to-green-400

                theme3:text-slate-950
                theme3:drop-shadow-lg
                "#,
                Link { to: Routes::Home {}, "Hangry Games" }
            }

            nav {
                class: "cinzel-font text-xl theme1:text-amber-500 theme2:text-green-800 theme3:text-slate-800",
                ul {
                    class: "flex flex-row gap-16",
                    li {
                        class: "px-2",
                        Link {
                            class: "theme3:hover:border-b-1",
                            to: Routes::Home {},
                            "Home"
                        }
                    }
                    li {
                        class: "px-2",
                        Link {
                            class: "theme3:hover:border-b-1",
                            to: Routes::GamesList {},
                            "Games"
                        }
                    }
                    li {
                        class: "relative group inline-block",
                        input {
                            id: "theme-switcher",
                            r#type: "checkbox",
                            class: "peer sr-only",
                        }
                        label {
                            class: r#"
                            px-2
                            cursor-pointer

                            theme1:peer-focus:bg-amber-500
                            theme1:peer-checked:bg-amber-500
                            theme1:group-hover:bg-amber-500
                            theme1:group-hover:text-red-900
                            theme1:peer-focus:text-red-900
                            theme1:peer-checked:text-red-900

                            theme2:group-hover:bg-teal-500
                            theme2:group-hover:text-green-200
                            theme2:peer-focus:bg-teal-500
                            theme2:peer-focus:text-green-200
                            theme2:peer-checked:bg-teal-500
                            theme2:peer-checked:text-green-200

                            theme3:group-hover:border-b-1
                            theme3:peer-focus:border-b-1
                            theme3:peer-checked:border-b-1
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
                                peer-checked:opacity-100
                                peer-checked:visible
                                peer-focus:opacity-100
                                peer-focus:visible
                                theme1:bg-linear-to-b
                                theme1:from-amber-500
                                theme1:to-amber-700
                                theme2:bg-teal-500
                                theme3:bg-slate-600
                                theme3:border
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

