use dioxus::prelude::*;

#[component]
pub fn MapPinIcon(class: String) -> Element {
    rsx! {
        svg {
            // Removed fill, stroke, and stroke-width - these should be handled by the passed CSS class
            view_box: "0 0 24 24",
            class: "{class}", // Pass the provided class string to the SVG element
            path {
                fill_rule: "evenodd",
                clip_rule: "evenodd",
                d: "m11.54 22.351.07.04.028.016a.76.76 0 0 0 .723 0l.028-.015.071-.041a16.975 16.975 0 0 0 1.144-.742 19.58 19.58 0 0 0 2.683-2.282c1.944-1.99 3.963-4.98 3.963-8.827a8.25 8.25 0 0 0-16.5 0c0 3.846 2.02 6.837 3.963 8.827a19.58 19.58 0 0 0 2.682 2.282 16.975 16.975 0 0 0 1.145.742ZM12 13.5a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"
            }
        }
    }
}

