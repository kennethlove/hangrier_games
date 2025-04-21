use crate::components::icons::mockingjay::Mockingjay;
use crate::components::icons::mockingjay_arrow::MockingjayArrow;
use crate::components::icons::mockingjay_flight::MockingjayFlight;
use crate::components::Button;
use crate::routes::Routes;
use crate::storage::{use_persistent, AppState, Colorscheme};
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    let mut storage = use_persistent("hangry-games", AppState::default);
    let mut theme_signal: Signal<Colorscheme> = use_context();

    rsx! {
        header {
            class: r#"
            flex
            flex-col
            flex-wrap
            items-center
            "#,

            h1 {
                class: r#"
                pb-2

                theme1:font-[Cinzel]
                theme1:font-bold
                theme1:text-5xl
                theme1:md:text-7xl
                theme1:bg-radial
                theme1:bg-clip-text
                theme1:text-transparent
                theme1:from-amber-300
                theme1:to-red-500
                theme1:drop-shadow-sm/25

                theme2:font-[Playfair_Display]
                theme2:text-6xl
                theme2:text-transparent
                theme2:md:text-7xl
                theme2:bg-linear-to-b
                theme2:bg-clip-text
                theme2:from-teal-500
                theme2:to-green-400
                theme2:drop-shadow-sm/25

                theme3:text-5xl
                theme3:md:text-6xl
                theme3:bg-clip-text
                theme3:text-transparent
                theme3:bg-gold-rich
                theme3:font-[Orbitron]
                theme3:font-semibold
                theme3:drop-shadow-sm/25
                "#,

                "Hangry Games"
            }

            nav {
                aria_label: "Main navigation",
                class: r#"
                text-lg
                sm:text-xl

                theme1:font-[Cinzel]

                theme2:text-md
                theme2:uppercase

                theme3:text-slate-800
                theme3:uppercase
                theme3:mt-2
                "#,

                ul {
                    class: "flex flex-row flex-grow gap-8",
                    li {
                        Link {
                            class: r#"
                            theme1:hover:bg-amber-500
                            theme1:text-amber-500
                            theme1:hover:text-amber-900
                            theme1:font-semibold
                            theme1:px-2

                            theme2:text-green-200/50
                            theme2:hover:text-green-200
                            theme2:hover:underline
                            theme2:hover:decoration-wavy
                            theme2:hover:decoration-2

                            theme3:transform
                            theme3:duration-500
                            theme3:text-yellow-600
                            theme3:border-b-5
                            theme3:border-transparent
                            theme3:border-double
                            theme3:hover:border-b-5
                            theme3:hover:border-yellow-500
                            theme3:hover:text-yellow-500
                            "#,

                            to: Routes::Home {},
                            "Home"
                        }
                    }
                    li {
                        Link {
                            class: r#"
                            theme1:hover:bg-amber-500
                            theme1:text-amber-500
                            theme1:hover:text-amber-900
                            theme1:font-semibold
                            theme1:px-2

                            theme2:text-green-200/50
                            theme2:hover:text-green-200
                            theme2:hover:underline
                            theme2:hover:decoration-wavy
                            theme2:hover:decoration-2

                            theme3:transform
                            theme3:duration-500
                            theme3:text-yellow-600
                            theme3:border-b-5
                            theme3:border-transparent
                            theme3:border-double
                            theme3:hover:border-b-5
                            theme3:hover:border-yellow-500
                            theme3:hover:text-yellow-500
                            "#,

                            to: Routes::GamesList {},
                            "Games"
                        }
                    }
                    li {
                        class: "relative group inline-block",
                        Link {
                            class: r#"
                            theme1:hover:bg-amber-500
                            theme1:text-amber-500
                            theme1:hover:text-amber-900
                            theme1:font-semibold
                            theme1:px-2

                            theme2:text-green-200/50
                            theme2:hover:text-green-200
                            theme2:hover:underline
                            theme2:hover:decoration-wavy
                            theme2:hover:decoration-2

                            theme3:transform
                            theme3:duration-500
                            theme3:text-yellow-600
                            theme3:border-b-5
                            theme3:border-transparent
                            theme3:border-double
                            theme3:hover:border-b-5
                            theme3:hover:border-yellow-500
                            theme3:hover:text-yellow-500
                            "#,
                            to: Routes::AccountsPage {},
                            "Account"
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

                            theme1:font-semibold
                            theme1:text-xl
                            theme1:text-amber-500
                            theme1:group-focus:bg-amber-500
                            theme1:group-focus:text-red-900
                            theme1:group-focus:border-b-2
                            theme1:group-focus:border-amber-500
                            theme1:focus-within:bg-amber-500
                            theme1:focus-within:text-red-900
                            theme1:focus-within:border-b-2
                            theme1:focus-within:border-amber-500

                            theme2:text-green-200/50
                            theme2:hover:text-green-900
                            theme2:peer-focus:bg-green-200
                            theme2:peer-focus:text-green-900
                            theme2:peer-focus:rounded-t-sm
                            theme2:peer-focus:border-b-3
                            theme2:peer-focus:border-green-200
                            theme2:focus-within:bg-green-200
                            theme2:focus-within:text-green-900
                            theme2:focus-within:rounded-t-sm
                            theme2:focus-within:border-b-3
                            theme2:focus-within:border-green-200

                            theme3:transform
                            theme3:duration-500
                            theme3:text-yellow-600
                            theme3:border-b-5
                            theme3:border-transparent
                            theme3:border-double
                            theme3:hover:border-b-5
                            theme3:hover:border-yellow-500
                            theme3:hover:text-yellow-500
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
                                invisible
                                w-64
                                transform
                                duration-500
                                peer-focus:opacity-100
                                peer-focus:visible
                                focus-within:opacity-100
                                focus-within:visible

                                theme1:bg-linear-to-b
                                theme1:from-amber-500
                                theme1:to-amber-700

                                theme2:bg-green-200
                                theme2:rounded-sm
                                theme2:rounded-tl-none

                                theme3:bg-stone-50
                                theme3:border
                                theme3:border-5
                                theme3:border-gold-rich
                                theme3:box-decoration-clone
                            "#,

                            div {
                                class: "grid grid-cols-3 place-content-center gap-2 pr-4",
                                label {
                                    title: "Switch to theme 1",
                                    class: "px-2 py-1 cursor-pointer size-24 border-none theme1:hover:cursor-not-allowed",
                                    onclick: move |_| {
                                        let mut state = storage.get();
                                        state.switch_to_theme_one();
                                        storage.set(state.clone());
                                        theme_signal.set(state.colorscheme)
                                    },
                                    input {
                                        class: "sr-only peer",
                                        r#type: "radio",
                                        name: "theme",
                                        value: "theme1",
                                        checked: storage.get().colorscheme == Colorscheme::One,
                                    }
                                    MockingjayArrow { class: r#"
                                    stroke-50
                                    fill-red-700
                                    stroke-red-900
                                    hover:stroke-red-500
                                    theme1:hover:stroke-red-900
                                    theme1:stroke-red-900
                                    peer-checked:stroke-red-500
                                    peer-focus:stroke-red-500
                                    "# }
                                }
                                label {
                                    title: "Switch to theme 2",
                                    class: "px-2 py-1 cursor-pointer size-24 border-none theme2:hover:cursor-not-allowed",
                                    onclick: move |_| {
                                        let mut state = storage.get();
                                        state.switch_to_theme_two();
                                        storage.set(state.clone());
                                        theme_signal.set(state.colorscheme)
                                    },
                                    input {
                                        class: "sr-only peer",
                                        r#type: "radio",
                                        name: "theme",
                                        value: "theme2",
                                        checked: storage.get().colorscheme == Colorscheme::Two,
                                    }
                                    Mockingjay { class: r#"
                                    stroke-50
                                    fill-green-700
                                    stroke-green-900
                                    hover:stroke-green-200
                                    theme2:hover:stroke-green-900
                                    theme2:stroke-green-900
                                    peer-checked:stroke-green-200
                                    peer-focus:stroke-green-200
                                    "# }
                                }
                                label {
                                    title: "Switch to theme 3",
                                    class: "px-2 py-1 cursor-pointer size-24 theme3:hover:cursor-not-allowed",
                                    onclick: move |_| {
                                        let mut state = storage.get();
                                        state.switch_to_theme_three();
                                        storage.set(state.clone());
                                        theme_signal.set(state.colorscheme)
                                    },
                                    input {
                                        class: "peer sr-only",
                                        r#type: "radio",
                                        name: "theme",
                                        value: "theme3",
                                        checked: storage.get().colorscheme == Colorscheme::Three,
                                    }
                                    MockingjayFlight {class: r#"
                                    stroke-50
                                    fill-amber-500
                                    stroke-amber-700
                                    hover:stroke-amber-200
                                    theme3:stroke-amber-200
                                    theme3:fill-amber-500
                                    theme3:hover:stroke-amber-700
                                    theme3:hover:fill-amber-500
                                    "# }
                                }
                            }
                        }
                    }
                }
            }
        }
        main {
            class: r#"
            mx-auto
            max-w-3/4
            "#,
            Outlet::<Routes> {}
        }
    }
}
