use dioxus::prelude::*;

#[component]
pub fn InfectionIcon(class: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 512 512",
            class,
            path {
                d: "M233.656 22.094c-13.884.19-28.38 2.95-42.97 8.843 30 .765 65.91 7.887 84.97 31.22 8.688 10.636 11.745 27.18 10 44.062 ...existing path data...",
            }
        }
    }
}
