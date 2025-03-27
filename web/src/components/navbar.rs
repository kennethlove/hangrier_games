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

    let future = use_resource(move || async move {
        let mut eval = document::eval(
            r#"
            "#,
        );
    });

    rsx! {
        header {
            // class: "flex flex-row place-content-between mb-4",
            class: "flex flex-col flex-wrap items-center",
            h1 {
                class: "text-5xl cinzel-font text-amber-500",
                Link { to: Routes::Home {}, "Hangry Games" }
            }

            nav {
                class: "cinzel-font text-amber-500 text-xl",
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
                        button {
                            class: "px-2 border border-transparent group-hover:bg-amber-500 theme2:group-hover:text-green-900 theme1:group-hover:text-red-900 theme3:group-hover:text-blue-900",
                            "Theme"
                        }
                        div {
                            class: "absolute z-99 opacity-0 open:opacity-100 w-64 invisible group-hover:opacity-100 group-hover:visible bg-amber-500",

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
                            // onclick: move |_| {
                            //     let mut state = storage.get();
                            //     state.toggle_dark_mode();
                            //     storage.set(state.clone());
                            //     dark_mode_signal.set(state.dark_mode);
                            // },
                        // }
                    }
                }
            }
        }
        Outlet::<Routes> {}
    }
}

