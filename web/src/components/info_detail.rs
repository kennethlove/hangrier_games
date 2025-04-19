use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Props)]
pub struct InfoDetailProps {
    pub title: String,
    pub open: bool,
    pub children: Element,
}

#[component]
pub fn InfoDetail(props: InfoDetailProps) -> Element {
    rsx! {
        details {
            open: props.open,
            class: r#"
            px-2
            pt-1
            open:pb-2
            group
            transition
            duration-500
            self-start

            theme1:bg-stone-800/50
            theme1:hover:bg-stone-800
            theme1:open:bg-stone-800/50

            theme2:bg-green-900
            theme2:rounded-md
            theme2:border
            theme2:border-green-800
            theme2:hover:border-green-400
            theme2:open:border-green-400

            theme3:bg-stone-50/80
            theme3:border-4
            theme3:border-gold-rich
            "#,

            summary {
                class: r#"
                flex
                items-center
                justify-between
                cursor-pointer
                "#,

                h3 {
                    class: r#"
                    mb-2
                    transition

                    theme1:text-xl
                    theme1:font-[Cinzel]
                    theme1:text-amber-300/75
                    theme1:group-open:text-amber-300
                    theme1:hover:text-amber-300

                    theme2:font-[Forum]
                    theme2:text-2xl
                    theme2:text-green-200
                    theme2:group-open:text-green-400

                    theme3:font-[Orbitron]
                    theme3:tracking-wider
                    "#,

                    "{props.title}",
                }
                span {
                    class: "transition duration-500 group-open:rotate-180",
                    svg {
                        class: r#"
                        size-4
                        fill-none
                        stroke-current

                        theme1:stroke-amber-300
                        theme1:hover:stroke-amber-300
                        theme1:group-open:stroke-amber-300

                        theme2:group-open:stroke-green-400
                        theme2:stroke-green-200
                        theme2:hover:stroke-green-400
                        "#,
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M19 9l-7 7-7-7"
                        }
                    }
                }
            }
            {props.children}
        }
    }
}
