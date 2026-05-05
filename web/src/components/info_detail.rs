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
            class: "px-3 pt-2 open:pb-2 group transition duration-500 self-start \
                    bg-surface border border-border rounded-card hover:border-primary open:border-primary",

            summary {
                class: "flex items-center justify-between cursor-pointer",
                h3 {
                    class: "mb-2 font-display text-xl tracking-wide text-text-muted group-open:text-text hover:text-text transition",
                    "{props.title}",
                }
                span {
                    class: "transition duration-500 group-open:rotate-180",
                    svg {
                        class: "size-4 fill-none stroke-current text-text-muted group-open:text-primary",
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
