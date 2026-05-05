use dioxus::prelude::*;

#[component]
pub fn LivePill() -> Element {
    rsx! {
        span {
            class: "inline-flex items-center gap-1.5 px-2.5 py-0.5 bg-danger text-white \
                    font-text font-bold text-[10px] uppercase tracking-[0.16em] rounded-sm",
            span { class: "size-1.5 rounded-full bg-white", " " }
            "Live"
        }
    }
}
